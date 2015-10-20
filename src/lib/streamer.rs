//! Tool for generating on-chip data.

#[cfg(test)]
extern crate assert;

#[macro_use]
extern crate log;

extern crate configuration;
extern crate fractal;
extern crate probability;
extern crate random;
extern crate sql;
extern crate sqlite;
extern crate temperature;
extern crate threed_ice;

#[macro_use]
mod macros;

mod result;

pub mod platform;
pub mod schedule;
pub mod system;
pub mod traffic;
pub mod workload;

pub use result::{Error, Result};

/// An outcome.
pub type Outcome<T> = Result<Option<T>>;

/// A configuration.
pub type Config = configuration::Tree;

/// A source of randomness.
pub type Source = random::Default;

mod math {
    #[link_name = "m"]
    extern {
        fn nextafter(x: f64, y: f64) -> f64;
    }

    #[inline]
    pub fn next_after(x: f64) -> f64 {
        use std::f64::INFINITY;
        unsafe { nextafter(x, INFINITY) }
    }
}
