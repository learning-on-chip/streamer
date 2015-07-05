use rustc_serialize::{Decodable, Decoder};
use std::fs::File;
use std::path::Path;
use toml::{Table, Value};

use Result;

pub struct Details(pub Value);

pub trait Detailable {
    fn set_details<'l>(&mut self, Option<&'l Value>);
    fn get_details<'l>(&'l self) -> Option<&'l Value>;

    fn detail(&self, name: &str) -> Option<String> {
        self.get_details().and_then(|value| {
            match value.lookup(name) {
                Some(&Value::String(ref string)) => Some(string.clone()),
                _ => None,
            }
        })
    }
}

#[derive(RustcDecodable)]
pub struct Config {
    pub traffic: Option<Traffic>,
    pub workload: Option<Workload>,
    pub details: Option<Details>,
}

#[derive(RustcDecodable)]
pub struct Traffic {
    pub path: Option<String>,
    pub query: Option<String>,
    pub details: Option<Details>,
}

#[derive(RustcDecodable)]
pub struct Workload {
    pub sources: Option<Vec<Source>>,
    pub details: Option<Details>,
}

#[derive(RustcDecodable)]
pub struct Source {
    pub name: Option<String>,
    pub path: Option<String>,
    pub details: Option<Details>,
}

impl Config {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Config> {
        use std::io::Read;

        let path = path.as_ref();
        let mut content = String::new();
        ok!(ok!(File::open(path)).read_to_string(&mut content));

        Config::parse(&content, |table| {
            if let Some(root) = path.parent() {
                if let Some(root) = root.to_str() {
                    table.insert("root".to_string(), Value::String(root.to_string()));
                }
            }
        })
    }

    fn parse<F>(content: &str, broadcast: F) -> Result<Config> where F: FnOnce(&mut Table) {
        use std::mem::replace;
        use toml::{Decoder, Parser};

        let mut parser = Parser::new(content);
        let config = match parser.parse() {
            Some(config) => {
                let mut decoder = Decoder::new(Value::Table(config));
                let mut config: Config = ok!(Decodable::decode(&mut decoder));
                let mut table = match replace(&mut decoder.toml, None) {
                    Some(Value::Table(table)) => table,
                    _ => Table::new(),
                };
                broadcast(&mut table);
                config.set_details(Some(&Value::Table(table)));
                config
            },
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

        Ok(config)
    }
}

impl Decodable for Details {
    #[inline]
    fn decode<D: Decoder>(_: &mut D) -> ::std::result::Result<Self, D::Error> {
        panic!("“details” is a reserved keyword");
    }
}

macro_rules! implement(
    ($kind:ty, [$($scalar:ident),*], [$($vector:ident),*]) => (
        impl Detailable for $kind {
            #[allow(unused_imports, unused_mut, unused_variables)]
            fn set_details<'l>(&mut self, value: Option<&'l Value>) {
                use std::mem::replace;

                let value = match value {
                    Some(value) => value,
                    _ => {
                        self.details = None;
                        return;
                    },
                };

                let broadcast = match value {
                    &Value::Table(ref table) => {
                        let mut table = table.clone();
                        $(table.remove(stringify!($scalar));)*
                        $(table.remove(stringify!($vector));)*
                        Some(Value::Table(table))
                    },
                    _ => None,
                };
                let broadcast = broadcast.as_ref();

                $(
                    if let Some(ref mut child) = self.$scalar {
                        let value = merge(broadcast, value.lookup(stringify!($scalar)));
                        child.set_details(value.as_ref());
                    }
                )*
                $(
                    if let Some(ref mut children) = self.$vector {
                        if let Some(&Value::Array(ref array)) = value.lookup(stringify!($vector)) {
                            for (child, value) in children.iter_mut().zip(array) {
                                let value = merge(broadcast, Some(value));
                                child.set_details(value.as_ref());
                            }
                        } else {
                            for child in children.iter_mut() {
                                child.set_details(broadcast);
                            }
                        }
                    }
                )*

                let value = merge(self.details.as_ref().map(|details| &details.0), Some(value));
                self.details = value.map(|value| Details(value));
            }

            #[inline]
            fn get_details<'l>(&'l self) -> Option<&'l Value> {
                self.details.as_ref().map(|details| &details.0)
            }
        }
    );
    ($kind:ty, [$($scalar:ident),*]) => (
        implement!($kind, [$($scalar),*], []);
    );
    ($kind:ty) => (
        implement!($kind, [], []);
    );
);

implement!(Config, [traffic, workload]);

implement!(Traffic);

implement!(Workload, [], [sources]);
implement!(Source);

fn merge<'l>(into: Option<&'l Value>, from: Option<&'l Value>) -> Option<Value> {
    use toml::Value::Table;

    match (into, from) {
        (Some(&Value::Table(ref into)), Some(&Value::Table(ref from))) => {
            let mut table = into.clone();
            for (key, value) in from {
                table.insert(key.clone(), value.clone());
            }
            Some(Value::Table(table))
        },
        (Some(value), Some(&Value::Integer(0))) => Some(value.clone()),
        (Some(value), None) => Some(value.clone()),
        (None, Some(value)) => Some(value.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, Detailable};
    use toml::Value;

    #[test]
    fn parse() {
        let content = r#"
            [traffic]
            path = "traffic/path"

            [[workload.sources]]
            path = "workload1/path"

            [[workload.sources]]
            path = "workload2/path"
            root = "bar"
        "#;
        let config = Config::parse(content, |table| {
            table.insert("root".to_string(), Value::String("foo".to_string()));
        }).unwrap();

        assert_eq!(&**config.traffic.as_ref().unwrap().path.as_ref().unwrap(), "traffic/path");
        assert_eq!(&config.traffic.as_ref().unwrap().detail("root").unwrap(), "foo");
        assert_eq!(&config.workload.as_ref().unwrap().detail("root").unwrap(), "foo");

        let sources = config.workload.as_ref().unwrap().sources.as_ref().unwrap();

        assert_eq!(&**sources[0].path.as_ref().unwrap(), "workload1/path");
        assert_eq!(&**sources[1].path.as_ref().unwrap(), "workload2/path");
        assert_eq!(&sources[0].detail("root").unwrap(), "foo");
        assert_eq!(&sources[1].detail("root").unwrap(), "bar");
    }
}
