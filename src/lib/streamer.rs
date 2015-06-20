extern crate rustc_serialize;
extern crate sqlite;
extern crate toml;

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

mod config;
mod source;

use config::Config;
use source::Source;

pub type Error = Box<std::fmt::Display>;
pub type Result<T> = std::result::Result<T, Error>;

pub struct Streamer {
    sources: Vec<source::Source>,
}

pub struct Stream<'l> {
    streamer: &'l Streamer,
}

pub struct State;

impl Streamer {
    pub fn new<T: AsRef<Path>>(config: T) -> Result<Streamer> {
        let root = config.as_ref().parent().map(|root| PathBuf::from(root))
                                           .unwrap_or(PathBuf::new());
        let config = try!(Config::new(config));

        let mut sources = vec![];
        match config.power.and_then(|config| config.sources) {
            Some(ref configs) => for config in configs {
                sources.push(try!(Source::new(config, &root)));
            },
            _ => {},
        }
        if sources.is_empty() {
            raise!("at least one power source is required");
        }

        Ok(Streamer {
            sources: sources,
        })
    }

    #[inline]
    pub fn iter<'l>(&'l self) -> Stream<'l> {
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
