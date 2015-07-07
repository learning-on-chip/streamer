use sqlite::{Connection, State};
use std::collections::HashMap;

use {Random, Result};
use config::Config;

pub struct Workload {
    pub patterns: Vec<Pattern>,
}

pub struct Pattern {
    pub components: Vec<Component>,
}

pub struct Component {
    pub name: String,
    pub dynamic_power: Vec<f64>,
    pub leakage_power: f64,
}

impl Workload {
    pub fn new(config: &Config, _: &Random) -> Result<Workload> {
        let mut patterns = vec![];
        if let Some(configs) = config.get::<Vec<Config>>("patterns") {
            for config in configs {
                patterns.push(try!(Pattern::new(config)));
            }
        }
        if patterns.is_empty() {
            raise!("at least one workload pattern is required");
        }
        Ok(Workload { patterns: patterns })
    }
}

impl Pattern {
    pub fn new(config: &Config) -> Result<Pattern> {
        let backend = ok!(Connection::open(&path!(config, "a workload pattern database")));

        info!(target: "workload", "Reading a database...");

        let mut names = match config.get::<String>("query_names") {
            Some(query) => try!(read_names(&backend, query)),
            _ => raise!("an SQL query for reading components’ names is required"),
        };
        let mut dynamic_power = match config.get::<String>("query_dynamic_power") {
            Some(query) => try!(read_dynamic_power(&backend, query)),
            _ => raise!("an SQL query for reading the dynamic power is required"),
        };
        let mut leakage_power = match config.get::<String>("query_leakage_power") {
            Some(query) => try!(read_leakage_power(&backend, query)),
            _ => raise!("an SQL query for reading the leakage power is required"),
        };

        let mut ids = names.keys().map(|&name| name).collect::<Vec<_>>();
        ids.sort();

        let mut components = vec![];
        for i in ids {
            components.push(Component {
                name: names.remove(&i).unwrap(),
                dynamic_power: match dynamic_power.remove(&i) {
                    Some(value) => value,
                    _ => raise!("cannot find the dynamic power of a component"),
                },
                leakage_power: match leakage_power.remove(&i) {
                    Some(value) => value,
                    _ => raise!("cannot find the leakage power of a component"),
                },
            });
        }

        Ok(Pattern { components: components })
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
        let backend = Connection::open("tests/fixtures/blackscholes.sqlite3").unwrap();
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
        let backend = Connection::open("tests/fixtures/blackscholes.sqlite3").unwrap();
        let data = super::read_dynamic_power(&backend, "
            SELECT `component_id`, `dynamic_power` FROM `dynamic`
            ORDER BY `time` ASC;
        ").unwrap();

        assert_eq!(data.len(), 2 + 1);
        for (_, data) in &data {
            assert_eq!(data.len(), 76);
        }
        assert::close(&[data.get(&0).unwrap()[2]], &[0.608065803127267], 1e-14);
        assert::close(&[data.get(&1).unwrap()[4]], &[9.19824419508802], 1e-14);
        assert::close(&[data.get(&2).unwrap()[0]], &[0.00613680976814029], 1e-14);
    }

    #[test]
    fn read_leakage_power() {
        let backend = Connection::open("tests/fixtures/blackscholes.sqlite3").unwrap();
        let data = super::read_leakage_power(&backend, "
            SELECT `component_id`, `leakage_power` FROM `static`;
        ").unwrap();

        assert_eq!(data.len(), 2 + 1);
        assert_eq!(data.get(&0).unwrap(), data.get(&1).unwrap());
    }
}
