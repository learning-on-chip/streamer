use sqlite::{Connection, State};

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
    pub fn new(config: &config::Workload) -> Result<Workload> {
        let mut sources = vec![];
        if let Some(ref configs) = config.sources {
            for config in configs {
                sources.push(try!(Source::new(config)));
            }
        }
        if sources.is_empty() {
            raise!("at least one workload source is required");
        }
        Ok(Workload { sources: sources })
    }
}

impl Source {
    pub fn new(config: &config::Source) -> Result<Source> {
        let backend = ok!(Connection::open(&path!(config, "a workload-source database")));
        info!(target: "workload", "Reading a database...");
        Ok(Source { components: try!(read_components(&backend)) })
    }
}

fn read_components(backend: &Connection) -> Result<Vec<Component>> {
    use std::collections::HashMap;

    let mut names = HashMap::new();
    let mut dynamic_power = HashMap::new();
    let mut leakage_power = HashMap::new();

    {
        let mut statement = ok!(backend.prepare("
            SELECT `component_id`, `name` FROM `static`;
        "));
        while State::Row == ok!(statement.step()) {
            let id = ok!(statement.read::<i64>(0));
            names.insert(id, ok!(statement.read::<String>(1)));
            dynamic_power.insert(id, vec![]);
            leakage_power.insert(id, 0.0);
        }
    }

    {
        let mut statement = ok!(backend.prepare("
            SELECT `component_id`, `dynamic_power` FROM `dynamic` ORDER BY `time` ASC;
        "));
        while State::Row == ok!(statement.step()) {
            match dynamic_power.get_mut(&ok!(statement.read::<i64>(0))) {
                Some(value) => value.push(ok!(statement.read::<f64>(1))),
                _ => raise!("found a dynamic-power value with an unknown ID"),
            }
        }
    }

    {
        let mut statement = ok!(backend.prepare("
            SELECT `component_id`, `leakage_power` FROM `static`;
        "));
        while State::Row == ok!(statement.step()) {
            match leakage_power.get_mut(&ok!(statement.read::<i64>(0))) {
                Some(value) => *value = ok!(statement.read::<f64>(1)),
                _ => raise!("found a leakage-power value with an unknown ID"),
            }
        }
    }

    let mut ids = names.keys().map(|&id| id).collect::<Vec<_>>();
    ids.sort();

    let mut components = vec![];
    for i in ids {
        components.push(Component {
            name: names.remove(&i).unwrap(),
            dynamic_power: dynamic_power.remove(&i).unwrap(),
            leakage_power: leakage_power.remove(&i).unwrap(),
        });
    }

    Ok(components)
}

#[cfg(test)]
mod tests {
    use assert;
    use sqlite::Connection;

    #[test]
    fn read_components() {
        let backend = Connection::open("tests/fixtures/blackscholes.sqlite3").unwrap();
        let components = super::read_components(&backend).ok().unwrap();
        assert_eq!(components.len(), 2 + 1);
        for component in components.iter() {
            assert_eq!(component.dynamic_power.len(), 76);
        }
        assert::close(&[components[0].dynamic_power[2]], &[0.608065803127267], 1e-14);
        assert::close(&[components[1].dynamic_power[4]], &[9.19824419508802], 1e-14);
        assert::close(&[components[2].dynamic_power[0]], &[0.00613680976814029], 1e-14);
    }
}
