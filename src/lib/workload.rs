use probability::distribution::{Categorical, Sample};
use sqlite::{Connection, State};
use std::collections::HashMap;
use std::rc::Rc;

use config::Config;
use {Result, Source};

pub struct Workload {
    patterns: Vec<Rc<Pattern>>,
    source: Source,
    distribution: Categorical,
}

pub struct Pattern {
    pub name: String,
    pub components: Vec<Component>,
}

pub struct Component {
    pub name: String,
    pub dynamic_power: Vec<f64>,
    pub leakage_power: f64,
}

impl Workload {
    pub fn new(config: &Config, source: &Source) -> Result<Workload> {
        let mut patterns = vec![];
        if let Some(ref configs) = config.collection("patterns") {
            for config in configs {
                patterns.push(Rc::new(try!(Pattern::new(config))));
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

    pub fn next(&mut self) -> Option<Rc<Pattern>> {
        Some(self.patterns[self.distribution.sample(&mut self.source)].clone())
    }
}

impl Pattern {
    pub fn new(config: &Config) -> Result<Pattern> {
        let path = path!(config, "a workload pattern database");
        let backend = ok!(Connection::open(&path));

        info!(target: "workload", "Reading {:?}...", &path);
        let name = match config.get::<String>("name") {
            Some(name) => name.to_string(),
            _ => path.file_stem().unwrap().to_str().unwrap().to_string(),
        };
        let mut names = {
            let query = some!(config.get::<String>("query_names"),
                              "an SQL query for reading components’ names is required");
            try!(read_names(&backend, query))
        };
        let mut dynamic_power = {
            let query = some!(config.get::<String>("query_dynamic_power"),
                              "an SQL query for reading the dynamic power is required");
            try!(read_dynamic_power(&backend, query))
        };
        let mut leakage_power = {
            let query = some!(config.get::<String>("query_leakage_power"),
                              "an SQL query for reading the leakage power is required");
            try!(read_leakage_power(&backend, query))
        };

        let mut ids = names.keys().map(|&id| id).collect::<Vec<_>>();
        ids.sort();

        let mut components = vec![];
        for id in ids {
            components.push(Component {
                name: names.remove(&id).unwrap(),
                dynamic_power: some!(dynamic_power.remove(&id),
                                     "cannot find the dynamic power of a component"),
                leakage_power: some!(leakage_power.remove(&id),
                                     "cannot find the leakage power of a component"),
            });
        }

        Ok(Pattern { name: name, components: components })
    }
}

fn read_names(backend: &Connection, query: &str) -> Result<HashMap<i64, String>> {
    let mut data = HashMap::new();
    let mut statement = ok!(backend.prepare(query));
    while State::Row == ok!(statement.step()) {
        data.insert(ok!(statement.read::<i64>(0)), ok!(statement.read::<String>(1)));
    }
    Ok(data)
}

fn read_dynamic_power(backend: &Connection, query: &str) -> Result<HashMap<i64, Vec<f64>>> {
    let mut data = HashMap::new();
    let mut statement = ok!(backend.prepare(query));
    while State::Row == ok!(statement.step()) {
        data.entry(ok!(statement.read::<i64>(0))).or_insert_with(|| vec![])
                                                 .push(ok!(statement.read::<f64>(1)));
    }
    Ok(data)
}

fn read_leakage_power(backend: &Connection, query: &str) -> Result<HashMap<i64, f64>> {
    let mut data = HashMap::new();
    let mut statement = ok!(backend.prepare(query));
    while State::Row == ok!(statement.step()) {
        data.insert(ok!(statement.read::<i64>(0)), ok!(statement.read::<f64>(1)));
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use assert;
    use sqlite::Connection;

    #[test]
    fn read_names() {
        let backend = Connection::open("tests/fixtures/parsec/blackscholes.sqlite3").unwrap();
        let data = super::read_names(&backend, "
            SELECT `component_id`, `name` FROM `static`;
        ").unwrap();

        assert_eq!(data.len(), 2 + 1);
        assert_eq!(data.get(&0).unwrap(), "core0");
        assert_eq!(data.get(&1).unwrap(), "core1");
        assert_eq!(data.get(&2).unwrap(), "l30");
    }

    #[test]
    fn read_dynamic_power() {
        let backend = Connection::open("tests/fixtures/parsec/blackscholes.sqlite3").unwrap();
        let data = super::read_dynamic_power(&backend, "
            SELECT `component_id`, `dynamic_power` FROM `dynamic`
            ORDER BY `time` ASC;
        ").unwrap();

        assert_eq!(data.len(), 2 + 1);
        for (_, data) in &data {
            assert_eq!(data.len(), 76);
        }
        assert::close(&[data.get(&0).unwrap()[2]], &[0.608065803127267], 1e-14);
        assert::close(&[data.get(&1).unwrap()[4]], &[9.19809606345627], 1e-14);
        assert::close(&[data.get(&2).unwrap()[0]], &[0.00613345574868796], 1e-14);
    }

    #[test]
    fn read_leakage_power() {
        let backend = Connection::open("tests/fixtures/parsec/blackscholes.sqlite3").unwrap();
        let data = super::read_leakage_power(&backend, "
            SELECT `component_id`, `leakage_power` FROM `static`;
        ").unwrap();

        assert_eq!(data.len(), 2 + 1);
        assert_eq!(data.get(&0).unwrap(), data.get(&1).unwrap());
    }
}
