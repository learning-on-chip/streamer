use options::Options;
use std::any::Any;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use toml::{self, Value};

use Result;

pub struct Config {
    node: Rc<Node>,
    path: String,
}

struct Node(Options);

impl Config {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Config> {
        use std::io::Read;

        let path = path.as_ref();
        let mut content = String::new();
        ok!(ok!(File::open(path)).read_to_string(&mut content));

        let mut node = try!(Node::parse(&content));
        if let Some(root) = path.parent() {
            node.set("root", root.to_path_buf());
        }

        Ok(Config {
            node: Rc::new(node),
            path: String::new(),
        })
    }

    pub fn get<'l, T: Any>(&'l self, path: &str) -> Option<&'l T> {
        let mut prefix = &*self.path;
        loop {
            if prefix.is_empty() {
                return self.node.lookup(path);
            }
            if let Some(value) = self.node.lookup(&format!("{}.{}", prefix, path)) {
                return Some(value);
            }
            prefix = match prefix.rfind('.') {
                Some(i) => &prefix[..i],
                _ => "",
            };
        }
    }

    pub fn branch(&self, path: &str) -> Option<Config> {
        if let None = self.get::<Node>(path) {
            return None;
        }
        Some(Config {
            node: self.node.clone(),
            path: if self.path.is_empty() {
                path.to_string()
            } else {
                format!("{}.{}", &self.path, path)
            },
        })
    }
}

impl Node {
    fn parse(content: &str) -> Result<Node> {
        use toml::Parser;

        let mut parser = Parser::new(content);
        let node = match parser.parse() {
            Some(node) => node,
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

        Node::from(node)
    }

    fn from(mut table: toml::Table) -> Result<Node> {
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
                            array.push(try!(Node::from(inner)));
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
                Value::Table(inner) => value.set(try!(Node::from(inner))),
            }
        }

        Ok(Node(options))
    }

    fn lookup<'l, T: Any>(&'l self, path: &str) -> Option<&'l T> {
        let chunks = path.split('.').collect::<Vec<_>>();
        let count = chunks.len();
        let mut current = self;
        let mut i = 0;
        while i < count {
            if i + 1 == count {
                return current.0.get_ref(chunks[i]);
            }
            if let Some(node) = current.0.get_ref::<Node>(chunks[i]) {
                i += 1;
                current = node;
            } else if let Some(array) = current.0.get_ref::<Vec<Node>>(chunks[i]) {
                i += 1;
                match chunks[i].parse::<usize>() {
                    Ok(j) => if i + 1 == count {
                        return Any::downcast_ref(&array[j]);
                    } else {
                        i += 1;
                        current = &array[j];
                    },
                    _ => return None,
                }
            } else {
                return None;
            }
        }
        unreachable!()
    }

    #[inline]
    fn set<T: Any>(&mut self, name: &str, value: T) {
        self.0.set(name, value);
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use super::{Config, Node};

    #[test]
    fn branch_get() {
        let content = r#"
            qux = 69

            [foo]
            bar = 42

            [[bar.baz]]
            qux = 42
        "#;
        let config = Config {
            node: Rc::new(Node::parse(content).unwrap()),
            path: String::new(),
        };

        {
            let config = config.branch("foo").unwrap();
            assert_eq!(config.get::<i64>("bar").unwrap(), &42);
            assert_eq!(config.get::<i64>("qux").unwrap(), &69);
        }
        {
            let config = config.branch("bar").unwrap();
            assert_eq!(config.get::<i64>("qux").unwrap(), &69);
        }
        {
            let config = config.branch("bar.baz.0").unwrap();
            assert_eq!(config.get::<i64>("qux").unwrap(), &42);
        }
    }
}
