use probability::distribution::{Categorical, Sample};
use sqlite::Connection;
use std::collections::HashMap;
use std::rc::Rc;

use platform;
use {Config, Result, Source};

pub struct Workload {
    patterns: Vec<Pattern>,
    source: Source,
    distribution: Categorical,
}

#[derive(Clone, Debug)]
pub struct Pattern(Rc<Content>);

deref! { Pattern::0 => Content }

#[derive(Clone, Debug)]
pub struct Content {
    pub name: String,
    pub units: usize,
    pub steps: usize,
    pub time_step: f64,
    pub elements: Vec<Element>,
}

#[derive(Clone, Debug)]
pub struct Element {
    pub kind: platform::Kind,
    pub dynamic_power: Vec<f64>,
    pub leakage_power: f64,
}

impl Workload {
    pub fn new(config: &Config, source: &Source) -> Result<Workload> {
        let mut patterns = vec![];
        if let Some(ref configs) = config.forest("patterns") {
            for config in configs {
                patterns.push(try!(Pattern::new(config)));
            }
        }
        let count = patterns.len();
        if count == 0 {
            raise!("at least one workload pattern is required");
        }
        Ok(Workload {
            patterns: patterns,
            source: source.clone(),
            distribution: Categorical::new(&vec![1.0 / count as f64; count]),
        })
    }

    pub fn next(&mut self) -> Option<Pattern> {
        Some(self.patterns[self.distribution.sample(&mut self.source)].clone())
    }
}

impl Pattern {
    pub fn new(config: &Config) -> Result<Pattern> {
        let path = path!(config, "a workload-pattern database is required");
        let backend = ok!(Connection::open(&path));

        info!(target: "Workload", "Reading {:?}...", &path);
        let name = match config.get::<String>("name") {
            Some(name) => name.to_string(),
            _ => path.file_stem().unwrap().to_str().unwrap().to_string(),
        };
        let time_step = *some!(config.get::<f64>("time_step"), "a time step is required");
        let mut names = try!(read_names(&backend));
        let mut dynamic_power = try!(read_dynamic_power(&backend));
        let mut leakage_power = try!(read_leakage_power(&backend));

        let mut ids = names.keys().map(|&id| id).collect::<Vec<_>>();
        ids.sort();

        let mut elements = vec![];
        for id in ids {
            elements.push(Element {
                kind: try!(names.remove(&id).unwrap().parse()),
                dynamic_power: some!(dynamic_power.remove(&id),
                                     "cannot find the dynamic power of a processing element"),
                leakage_power: some!(leakage_power.remove(&id),
                                     "cannot find the leakage power of a processing element"),
            });
        }

        let units = elements.len();
        if units == 0 {
            raise!("found a workload pattern without processing elements");
        }

        let steps = elements[0].dynamic_power.len();
        if steps == 0 {
            raise!("found a workload pattern without dynamic-power data");
        }

        Ok(Pattern(Rc::new(Content {
            name: name,
            units: units,
            steps: steps,
            time_step: time_step,
            elements: elements,
        })))
    }

    #[inline]
    pub fn duration(&self) -> f64 {
        self.steps as f64 * self.time_step
    }
}

impl Element {
    pub fn accept(&self, element: &platform::Element) -> bool {
        self.kind == element.kind
    }
}

fn read_names(backend: &Connection) -> Result<HashMap<i64, String>> {
    use sql::prelude::*;

    let mut data = HashMap::new();
    let statement = select_from("static").columns(&["component_id", "name"]);
    let mut cursor = ok!(backend.prepare(ok!(statement.compile()))).cursor();
    while let Some(row) = ok!(cursor.next()) {
        if let (Some(id), Some(value)) = (row[0].as_integer(), row[1].as_string()) {
            data.insert(id, value.to_string());
        } else {
            raise!("failed to read the names of processing elements");
        }
    }
    Ok(data)
}

fn read_dynamic_power(backend: &Connection) -> Result<HashMap<i64, Vec<f64>>> {
    use sql::prelude::*;

    let mut data = HashMap::new();
    let statement = select_from("dynamic").columns(&["time", "component_id", "dynamic_power"])
                                          .order_by(column("time").ascend());
    let mut cursor = ok!(backend.prepare(ok!(statement.compile()))).cursor();
    while let Some(row) = ok!(cursor.next()) {
        if let (Some(id), Some(value)) = (row[1].as_integer(), row[2].as_float()) {
            data.entry(id).or_insert_with(|| vec![]).push(value);
        } else {
            raise!("failed to read the dynamic power");
        }
    }
    Ok(data)
}

fn read_leakage_power(backend: &Connection) -> Result<HashMap<i64, f64>> {
    use sql::prelude::*;

    let mut data = HashMap::new();
    let statement = select_from("static").columns(&["component_id", "leakage_power"]);
    let mut cursor = ok!(backend.prepare(ok!(statement.compile()))).cursor();
    while let Some(row) = ok!(cursor.next()) {
        if let (Some(id), Some(value)) = (row[0].as_integer(), row[1].as_float()) {
            data.insert(id, value);
        } else {
            raise!("failed to read the leakage power");
        }
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use assert;
    use sqlite::Connection;

    #[test]
    fn read_names() {
        let backend = open();
        let data = super::read_names(&backend).unwrap();

        assert_eq!(data.len(), 2 + 1);
        assert_eq!(data.get(&0).unwrap(), "core0");
        assert_eq!(data.get(&1).unwrap(), "core1");
        assert_eq!(data.get(&2).unwrap(), "l30");
    }

    #[test]
    fn read_dynamic_power() {
        let backend = open();
        let data = super::read_dynamic_power(&backend).unwrap();

        assert_eq!(data.len(), 2 + 1);
        for (_, data) in &data {
            assert_eq!(data.len(), 69);
        }
        assert::close(&[data.get(&0).unwrap()[2]], &[0.60806580312727], 1e-14);
        assert::close(&[data.get(&1).unwrap()[4]], &[8.68983889250007], 1e-14);
        assert::close(&[data.get(&2).unwrap()[0]], &[0.00620192518435], 1e-14);
    }

    #[test]
    fn read_leakage_power() {
        let backend = open();
        let data = super::read_leakage_power(&backend).unwrap();

        assert_eq!(data.len(), 2 + 1);
        assert_eq!(data.get(&0).unwrap(), data.get(&1).unwrap());
    }

    fn open() -> Connection {
        Connection::open("tests/fixtures/blackscholes.sqlite3").unwrap()
    }
}
