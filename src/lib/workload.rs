use sqlite::{Database, State};
use std::path::Path;

use Result;
use config;

pub struct Workload {
    sources: Vec<Source>,
}

pub struct Source {
    pub names: Vec<String>,
    pub dynamic: Vec<f64>,
    pub leakage: Vec<f64>,
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

        let (names, leakage) = try!(read_names_and_leakage_power(&backend));
        let dynamic = try!(read_dynamic_power(&backend, &names));

        Ok(Source {
            names: names,
            dynamic: dynamic,
            leakage: leakage,
        })
    }
}

fn read_names_and_leakage_power(backend: &Database) -> Result<(Vec<String>, Vec<f64>)> {
    let mut names = vec![];
    let mut data = vec![];
    let mut statement = ok!(backend.prepare("
        SELECT `name`, `value` FROM `static` WHERE `name` LIKE '%_leakage_power';
    "));
    while State::Row == ok!(statement.step()) {
        names.push({
            let name = ok!(statement.read::<String>(0));
            String::from(&name[..name.find('_').unwrap()])
        });
        data.push(ok!(statement.read::<f64>(1)));
    }
    Ok((names, data))
}

fn read_dynamic_power(backend: &Database, names: &[String]) -> Result<Vec<f64>> {
    let count = names.len();
    let mut data = Vec::with_capacity(count);
    let fields = {
        let mut buffer = String::new();
        for name in names.iter() {
            if !buffer.is_empty() {
                buffer.push_str(", ");
            }
            buffer.push_str(&format!("`{}_dynamic_power`", name));
        }
        buffer
    };
    let mut statement = ok!(backend.prepare(&format!("
        SELECT {} FROM `dynamic` ORDER BY `time` ASC;
    ", &fields)));
    while State::Row == ok!(statement.step()) {
        for i in 0..count {
            data.push(ok!(statement.read::<f64>(i)));
        }
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
        assert_eq!(source.dynamic.len(), (2 + 1) * 76);
    }
}
