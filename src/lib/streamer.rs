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

pub type Error = Box<std::fmt::Display>;
pub type Result<T> = std::result::Result<T, Error>;

pub struct Streamer {
    sources: Vec<Box<source::Source>>,
}

impl Streamer {
    pub fn new(config: &Path) -> Result<Streamer> {
        let root = config.parent().map(|path| PathBuf::from(path));
        let config = try!(config::open(config));

        let mut sources: Vec<Box<source::Source>> = Vec::new();
        for &config::Source { ref kind, ref path, .. } in config.sources.iter() {
            let mut path = PathBuf::from(path);
            if path.is_relative() {
                if let Some(ref root) = root {
                    path = root.join(path);
                }
            }
            if std::fs::metadata(&path).is_err() {
                raise!("the source file {:?} does not exist", &path);
            }
            match &**kind {
                "sqlite3" => sources.push(Box::new(ok!(sqlite::open(&path)))),
                _ => raise!("the source kind {:?} is unknown", kind),
            }
        }
        if sources.is_empty() {
            raise!("at least one source is required");
        }

        Ok(Streamer {
            sources: sources,
        })
    }
}
