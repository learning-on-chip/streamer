#[cfg(test)]
extern crate assert;

extern crate fractal;
extern crate probability;
extern crate random;
extern crate rustc_serialize;
extern crate sqlite;
extern crate toml;

#[macro_use]
extern crate log;

use random::Source;
use std::{error, fmt};
use std::path::{Path, PathBuf};

macro_rules! raise(
    ($message:expr) => (return Err(Box::new(::ErrorString($message.to_string()))));
    ($($arg:tt)*) => (return Err(Box::new(::ErrorString(format!($($arg)*)))));
);

macro_rules! ok(
    ($result:expr) => (match $result {
        Ok(result) => result,
        Err(error) => return Err(Box::new(error)),
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
mod traffic;
mod workload;

use config::Config;
use traffic::{Traffic, Queue};
use workload::Workload;

pub struct ErrorString(pub String);
pub type Error = Box<std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

pub struct Streamer {
    pub traffic: Traffic,
    pub workload: Workload,
}

pub struct Stream<'l, S: Source + 'l> {
    queue: Queue<'l, S>,
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

        let workload = match config.workload {
            Some(ref workload) => try!(Workload::new(workload, &root)),
            _ => raise!("a workload configuration is required"),
        };

        Ok(Streamer {
            traffic: traffic,
            workload: workload,
        })
    }

    #[inline]
    pub fn iter<'l, S: Source>(&'l self, source: &'l mut S) -> Stream<'l, S> {
        Stream { queue: self.traffic.iter(source) }
    }
}

impl<'l, S: Source> Iterator for Stream<'l, S> {
    type Item = State;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.next().map(|time| State(time))
    }
}

impl fmt::Display for State {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:25.15e}", self.0)
    }
}

impl fmt::Debug for ErrorString {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ErrorString {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl error::Error for ErrorString {
    #[inline]
    fn description(&self) -> &str {
        &self.0
    }
}
