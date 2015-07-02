extern crate fractal;
extern crate probability;
extern crate rustc_serialize;
extern crate sqlite;
extern crate toml;

use probability::generator;
use std::fmt;
use std::path::{Path, PathBuf};

#[macro_export]
macro_rules! raise(
    ($error:expr) => (return Err(Box::new($error)));
    ($($arg:tt)*) => (raise!(format!($($arg)*)));
);

#[macro_export]
macro_rules! ok(
    ($result:expr) => (match $result {
        Ok(result) => result,
        Err(error) => raise!(error),
    });
);

macro_rules! path(
    ($path:expr, $root:expr, $destination:expr) => ({
        let mut path = match $path {
            Some(ref path) => ::std::path::PathBuf::from(path),
            _ => raise!("the path to {} is missing", $destination),
        };
        if path.is_relative() {
            path = $root.join(path);
        }
        if ::std::fs::metadata(&path).is_err() {
            raise!("the file {:?} does not exist", &path);
        }
        path
    });
);

mod config;
mod source;
mod traffic;

use config::Config;
use source::Source;
use traffic::Traffic;

pub type Error = Box<std::fmt::Display>;
pub type Result<T> = std::result::Result<T, Error>;

pub struct Streamer {
    traffic: Traffic,
    sources: Vec<source::Source>,
}

pub struct Stream<'l> {
    streamer: &'l Streamer,
}

#[derive(Clone, Copy)]
pub struct State(f64);

impl Streamer {
    pub fn new<T: AsRef<Path>>(config: T) -> Result<Streamer> {
        let root = config.as_ref().parent().map(|root| PathBuf::from(root))
                                           .unwrap_or(PathBuf::new());
        let config = try!(Config::new(config));

        let traffic = match config.traffic {
            Some(ref traffic) => try!(Traffic::new(traffic, &root)),
            _ => raise!("a traffic configuration is required"),
        };

        let mut sources = vec![];
        if let Some(ref configs) = config.power.and_then(|config| config.sources) {
            for config in configs {
                sources.push(try!(Source::new(config, &root)));
            }
        }
        if sources.is_empty() {
            raise!("at least one power source is required");
        }

        Ok(Streamer {
            traffic: traffic,
            sources: sources,
        })
    }

    #[inline]
    pub fn iter<'l>(&'l self) -> Stream<'l> {
        let mut generator = generator::default();
        generator.seed([0x12345678, 0x87654321]);
        Stream {
            streamer: self,
        }
    }
}

impl<'l> Iterator for Stream<'l> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl fmt::Display for State {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:25.15e}", self.0)
    }
}
