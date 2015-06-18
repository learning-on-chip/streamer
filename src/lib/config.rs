use rustc_serialize::{Decodable, Decoder};
use std::fs::File;
use std::path::Path;
use toml;

use Result;

pub struct Details(pub toml::Value);

pub trait Detailable {
    fn set_details<'l>(&mut self, Option<&'l toml::Value>);
    fn get_details<'l>(&'l self) -> Option<&'l toml::Value>;

    fn lookup(&self, name: &str) -> Option<String> {
        self.get_details().and_then(|toml| {
            match toml.lookup(name) {
                Some(&toml::Value::String(ref string)) => Some(string.clone()),
                _ => None,
            }
        })
    }
}

#[derive(RustcDecodable)]
pub struct Config {
    pub time: Option<Time>,
    pub power: Option<Power>,
    pub temperature: Option<Temperature>,
    pub details: Option<Details>,
}

#[derive(RustcDecodable)]
pub struct Time {
    pub period: Option<f64>,
    pub details: Option<Details>,
}

#[derive(RustcDecodable)]
pub struct Power {
    pub sources: Option<Vec<Source>>,
    pub details: Option<Details>,
}

#[derive(RustcDecodable)]
pub struct Temperature {
    pub stack: Option<String>,
    pub details: Option<Details>,
}

#[derive(RustcDecodable)]
pub struct Source {
    pub name: Option<String>,
    pub kind: Option<String>,
    pub details: Option<Details>,
}

impl Config {
    pub fn new(path: &Path) -> Result<Config> {
        use std::io::Read;

        let mut contents = String::new();
        ok!(ok!(File::open(path)).read_to_string(&mut contents));

        let config = match toml::Parser::new(&contents).parse() {
            Some(root) => {
                let mut decoder = toml::Decoder::new(toml::Value::Table(root));
                let mut config: Config = ok!(Decodable::decode(&mut decoder));
                config.set_details(decoder.toml.as_ref());
                config
            },
            _ => raise!("failed to parse the configuration file"),
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

macro_rules! detailable(
    ($kind:ty, [$($scalar:ident),*], [$($vector:ident),*]) => (
        impl Detailable for $kind {
            fn set_details<'l>(&mut self, toml: Option<&'l toml::Value>) {
                self.details = toml.and_then(|toml| {
                    $(
                        if let Some(ref mut child) = self.$scalar {
                            child.set_details(toml.lookup(stringify!($scalar)));
                        }
                    )*
                    $(
                        if let Some(ref mut children) = self.$vector {
                            match toml.lookup(stringify!($vector)) {
                                Some(&toml::Value::Array(ref array)) => {
                                    for (child, toml) in children.iter_mut().zip(array) {
                                        child.set_details(Some(toml));
                                    }
                                },
                                _ => {},
                            }
                        }
                    )*
                    Some(Details(toml.clone()))
                });
            }

            #[inline]
            fn get_details<'l>(&'l self) -> Option<&'l toml::Value> {
                self.details.as_ref().map(|details| &details.0)
            }
        }
    );
    ($kind:ty, [$($scalar:ident),*]) => (
        detailable!($kind, [$($scalar),*], []);
    );
    ($kind:ty) => (
        detailable!($kind, [], []);
    );
);

detailable!(Config, [time, power, temperature]);

detailable!(Time);

detailable!(Power, [], [sources]);
detailable!(Source);

detailable!(Temperature);
