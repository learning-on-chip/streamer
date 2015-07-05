use options::Options;
use std::any::Any;
use std::fs::File;
use std::path::Path;
use toml::{self, Value};

use Result;

pub struct Config(Options);

impl Config {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Config> {
        use std::io::Read;

        let path = path.as_ref();
        let mut content = String::new();
        ok!(ok!(File::open(path)).read_to_string(&mut content));

        let mut config = try!(Config::parse(&content));
        if let Some(root) = path.parent() {
            config.broadcast("root", root.to_path_buf());
        }

        Ok(config)
    }

    pub fn broadcast<T: Any + Clone>(&mut self, name: &str, value: T) {
        for (_, parameter) in &mut self.0 {
            if let Some(config) = parameter.get_mut::<Config>() {
                config.broadcast(name, value.clone());
                continue;
            }
            if let Some(array) = parameter.get_mut::<Vec<Config>>() {
                for config in array {
                    config.broadcast(name, value.clone());
                }
            }
        }
        self.0.set(name, value);
    }

    pub fn get<'l, T: Any>(&'l self, path: &str) -> Option<&'l T> {
        let chunks = path.split('.').collect::<Vec<_>>();

        let count = chunks.len();
        if count == 0 {
            return None;
        }

        let mut current = self;
        let mut i = 0;
        while i + 1 < count {
            if let Some(config) = current.0.get_ref::<Config>(chunks[i]) {
                current = config;
                i += 1;
            } else if let Some(array) = current.0.get_ref::<Vec<Config>>(chunks[i]) {
                if i + 2 < count {
                    match chunks[i + 1].parse::<usize>() {
                        Ok(j) => current = &array[j],
                        _ => return None,
                    }
                } else {
                    return None;
                }
                i += 2;
            } else {
                return None;
            }
        }

        current.0.get_ref(chunks[count - 1])
    }

    fn parse(content: &str) -> Result<Config> {
        use toml::Parser;

        let mut parser = Parser::new(content);
        let config = match parser.parse() {
            Some(config) => config,
            _ => {
                let mut errors = String::new();
                for error in parser.errors {
                    if !errors.is_empty() {
                        errors.push_str(", ");
                    }
                    errors.push_str(&format!("{}", error));
                }
                raise!("failed to parse the configuration file ({})", errors);
            },
        };

        Config::from(config)
    }

    fn from(mut table: toml::Table) -> Result<Config> {
        let mut options = Options::new();

        for (name, _) in &table {
            options.set(name, 0);
        }
        for (name, value) in &mut options {
            match table.remove(name).unwrap() {
                Value::Array(inner) => {
                    let mut array = vec![];
                    for inner in inner {
                        if let Value::Table(inner) = inner {
                            array.push(try!(Config::from(inner)));
                        } else {
                            raise!("extected a table");
                        }
                    }
                    value.set(array);
                },
                Value::Boolean(inner) => value.set(inner),
                Value::Datetime(inner) => value.set(inner),
                Value::Float(inner) => value.set(inner),
                Value::Integer(inner) => value.set(inner),
                Value::String(inner) => value.set(inner),
                Value::Table(inner) => value.set(try!(Config::from(inner))),
            }
        }

        Ok(Config(options))
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn broadcast() {
        let content = r#"
            [foo]
            bar = 42

            [[bar.baz]]
            qux = 42
        "#;
        let mut config = Config::parse(content).unwrap();
        config.broadcast("qux", 69);

        assert_eq!(config.get::<i32>("foo.qux").unwrap(), &69);
        assert_eq!(config.get::<i32>("bar.qux").unwrap(), &69);
        assert_eq!(config.get::<i32>("bar.baz.0.qux").unwrap(), &69);
    }
}
