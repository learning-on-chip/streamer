#[cfg(test)]
extern crate assert;

#[macro_use]
extern crate log;

extern crate fractal;
extern crate options;
extern crate probability;
extern crate random;
extern crate sql;
extern crate sqlite;
extern crate temperature;
extern crate threed_ice;
extern crate toml;

use std::{error, fmt};

mod math {
    use std::f64::INFINITY;

    #[link_name = "m"]
    extern {
        fn nextafter(x: f64, y: f64) -> f64;
    }

    #[inline]
    pub fn next_after(x: f64) -> f64 {
        unsafe { nextafter(x, INFINITY) }
    }
}

#[macro_use]
mod macros;

mod config;
mod platform;
mod profile;
mod schedule;
mod system;
mod traffic;
mod workload;

pub use config::Config;
pub use platform::Platform;
pub use profile::Profile;
pub use system::{Increment, Job, System};

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
