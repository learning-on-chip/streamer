use sqlite::{Database, State};
use std::path::Path;

use Result;
use config;

pub struct Workload {
    pub sources: Vec<Source>,
}

pub struct Source {
    pub components: Vec<Component>,
}

pub struct Component {
    pub name: String,
    pub dynamic_power: Vec<f64>,
    pub leakage_power: f64,
}

impl Workload {
    pub fn new<T: AsRef<Path>>(config: &config::Workload, root: T) -> Result<Workload> {
        let mut sources = vec![];
        if let Some(ref configs) = config.sources {
            for config in configs {
                sources.push(try!(Source::new(config, &root)));
            }
        }
        if sources.is_empty() {
            raise!("at least one workload source is required");
        }
        Ok(Workload { sources: sources })
    }
}

impl Source {
    pub fn new<T: AsRef<Path>>(config: &config::Source, root: T) -> Result<Source> {
        let backend = ok!(Database::open(&path!(config.path, root.as_ref(),
                                                "a workload-source database")));

        info!(target: "workload", "Reading a database...");
        Ok(Source { components: try!(read_components(&backend)) })
    }
}

fn read_components(backend: &Database) -> Result<Vec<Component>> {
    let mut components = vec![];
    let mut statement = ok!(backend.prepare("
        SELECT `component_id`, `name`, `leakage_power` FROM `static`;
    "));
    while State::Row == ok!(statement.step()) {
        let component_id = ok!(statement.read::<i64>(0));
        components.push(Component {
            name: ok!(statement.read::<String>(1)),
            dynamic_power: try!(read_dynamic_power(backend, component_id)),
            leakage_power: ok!(statement.read::<f64>(2)),
        });
    }
    Ok(components)
}

fn read_dynamic_power(backend: &Database, component_id: i64) -> Result<Vec<f64>> {
    let mut data = vec![];
    let mut statement = ok!(backend.prepare(&format!("
        SELECT `dynamic_power` FROM `dynamic` WHERE `component_id` = {} ORDER BY `time` ASC;
    ", component_id)));
    while State::Row == ok!(statement.step()) {
        data.push(ok!(statement.read::<f64>(0)));
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use config;

    #[test]
    fn new() {
        let config = config::Source {
            name: None,
            path: Some("tests/fixtures/blackscholes.sqlite3".to_string()),
            details: None,
        };
        let source = super::Source::new(&config, "").ok().unwrap();
        assert_eq!(source.components.len(), 2 + 1);
        for component in source.components.iter() {
            assert_eq!(component.dynamic_power.len(), 76);
        }
    }
}
