//! Generation of on-chip data.

extern crate configuration;
extern crate fractal;
extern crate probability;
extern crate random;
extern crate sql;
extern crate sqlite;
extern crate temperature;
extern crate threed_ice;

#[cfg(test)] extern crate assert;
#[macro_use] extern crate log;

#[macro_use] mod macros;

mod math;
mod result;

pub mod output;
pub mod platform;
pub mod schedule;
pub mod system;
pub mod traffic;
pub mod workload;

pub use result::{Error, Result};

/// A configuration.
pub type Config = configuration::Tree;

/// A source of randomness.
pub type Source = random::Default;

/// Create a source of randomness.
pub fn source(config: &Config) -> Source {
    let mut seed = config.get::<i64>("seed").map(|&seed| seed as u64).unwrap_or(0);
    if seed == 0 {
        seed = !0u64
    }
    random::default().seed([0x12345678 & seed, 0x87654321 & seed])
}
