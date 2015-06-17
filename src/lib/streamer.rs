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
    sources: Vec<Box<source::Source>>,
}

pub struct Stream<'l> {
    streamer: &'l Streamer,
}

pub struct State;

impl Streamer {
    pub fn new(config: &Path) -> Result<Streamer> {
        let root = config.parent().map(|root| PathBuf::from(root)).unwrap_or(PathBuf::new());
        let config = try!(Config::new(config));

        let mut sources = vec![];
        if let Some(ref configs) = config.sources {
            for config in configs {
                sources.push(try!(source::new(config, &root)));
            }
        }
        if sources.is_empty() {
            raise!("at least one source is required");
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
