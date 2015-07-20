#[cfg(test)]
extern crate assert;

#[macro_use]
extern crate log;

extern crate fractal;
extern crate options;
extern crate probability;
extern crate random;
extern crate sqlite;
extern crate temperature;
extern crate threed_ice;
extern crate toml;

use std::{error, fmt};

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

macro_rules! some(
    ($option:expr, $($arg:tt)*) => (match $option {
        Some(value) => value,
        _ => raise!($($arg)*),
    });
);

macro_rules! path(
    ($config:ident, $destination:expr) => ({
        let path = some!($config.get::<String>("path"), "the path to {} is missing", $destination);
        let mut path = ::std::path::PathBuf::from(path);
        if path.is_relative() {
            if let Some(ref root) = $config.get::<::std::path::PathBuf>("root") {
                path = root.join(path);
            }
        }
        if ::std::fs::metadata(&path).is_err() {
            raise!("the file {:?} does not exist", &path);
        }
        path
    });
);

mod config;
mod platform;
mod system;
mod traffic;
mod workload;

pub use system::{State, System};

pub type Error = Box<std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

pub struct ErrorString(pub String);

pub type Source = random::Default;

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
