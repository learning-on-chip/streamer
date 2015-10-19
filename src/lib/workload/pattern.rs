use sqlite::Connection;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use platform::{self, ElementKind};

use {Config, Result};

/// A workload pattern.
#[derive(Clone, Debug)]
pub struct Pattern(Rc<Content>);

deref! { Pattern::0 => Content }

/// The content of a workload pattern.
#[derive(Clone, Debug)]
pub struct Content {
    /// The name.
    pub name: String,
    /// The number of processing elements.
    pub units: usize,
    /// The number of time steps.
    pub steps: usize,
    /// The time step (sampling interval).
    pub time_step: f64,
    /// The processing elements.
    pub elements: Vec<Element>,
}

/// A processing element of a workload pattern.
#[derive(Clone, Debug)]
pub struct Element {
    /// The type.
    pub kind: ElementKind,
    /// The area.
    pub area: f64,
    /// The leakage power.
    pub leakage_power: f64,
    /// The dynamic power.
    pub dynamic_power: Vec<f64>,
}

impl Pattern {
    /// Create a pattern.
    pub fn new(config: &Config) -> Result<Pattern> {
        let path = path!(config, "a workload-pattern database is required");

        let name = match config.get::<String>("name") {
            Some(name) => name.to_string(),
            _ => path.file_stem().unwrap().to_str().unwrap().to_string(),
        };
        let time_step = *some!(config.get::<f64>("time_step"), "a time step is required");

        info!(target: "Workload", "Reading {:?}...", &path);
        let elements = try!(read(&path));

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

    /// Return the time duration.
    #[inline]
    pub fn duration(&self) -> f64 {
        self.steps as f64 * self.time_step
    }
}

impl Element {
    /// Check if the type matches the type of a processing element of a
    /// platform.
    pub fn accept(&self, element: &platform::Element) -> bool {
        self.kind == element.kind
    }
}

fn read(path: &Path) -> Result<Vec<Element>> {
    let backend = ok!(Connection::open(path));

    let mut names = try!(read_names(&backend));
    let mut areas = try!(read_static(&backend, "area"));
    let mut leakage_power = try!(read_static(&backend, "leakage_power"));
    let mut dynamic_power = try!(read_dynamic(&backend, "dynamic_power"));

    let mut ids = names.keys().map(|&id| id).collect::<Vec<_>>();
    ids.sort();

    let mut elements = vec![];
    for id in ids {
        elements.push(Element {
            kind: try!(names.remove(&id).unwrap().parse()),
            area: some!(areas.remove(&id), "cannot find the area of a processing element"),
            leakage_power: some!(leakage_power.remove(&id),
                                 "cannot find the leakage power of a processing element"),
            dynamic_power: some!(dynamic_power.remove(&id),
                                 "cannot find the dynamic power of a processing element"),
        });
    }

    Ok(elements)
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

fn read_static(backend: &Connection, name: &str) -> Result<HashMap<i64, f64>> {
    use sql::prelude::*;

    let mut data = HashMap::new();
    let statement = select_from("static").columns(&["component_id", name]);
    let mut cursor = ok!(backend.prepare(ok!(statement.compile()))).cursor();
    while let Some(row) = ok!(cursor.next()) {
        if let (Some(id), Some(value)) = (row[0].as_integer(), row[1].as_float()) {
            data.insert(id, value);
        } else {
            raise!("failed to read the {} column", name);
        }
    }
    Ok(data)
}

fn read_dynamic(backend: &Connection, name: &str) -> Result<HashMap<i64, Vec<f64>>> {
    use sql::prelude::*;

    let mut data = HashMap::new();
    let statement = select_from("dynamic").columns(&["time", "component_id", name])
                                          .order_by(column("time").ascend());
    let mut cursor = ok!(backend.prepare(ok!(statement.compile()))).cursor();
    while let Some(row) = ok!(cursor.next()) {
        if let (Some(id), Some(value)) = (row[1].as_integer(), row[2].as_float()) {
            data.entry(id).or_insert_with(|| vec![]).push(value);
        } else {
            raise!("failed to read the {} column", name);
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
    fn read_dynamic() {
        let backend = open();
        let data = super::read_dynamic(&backend, "dynamic_power").unwrap();

        assert_eq!(data.len(), 2 + 1);
        for (_, data) in &data {
            assert_eq!(data.len(), 69);
        }
        assert::close(&[data.get(&0).unwrap()[2]], &[0.60806580312727], 1e-14);
        assert::close(&[data.get(&1).unwrap()[4]], &[8.68983889250007], 1e-14);
        assert::close(&[data.get(&2).unwrap()[0]], &[0.00620192518435], 1e-14);
    }

    #[test]
    fn read_static() {
        let backend = open();
        let data = super::read_static(&backend, "leakage_power").unwrap();

        assert_eq!(data.len(), 2 + 1);
        assert_eq!(data.get(&0).unwrap(), data.get(&1).unwrap());
    }

    fn open() -> Connection {
        Connection::open("tests/fixtures/blackscholes.sqlite3").unwrap()
    }
}
