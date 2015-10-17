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

mod event;
mod result;
mod system;
mod traffic;
mod workload;

pub mod platform;
pub mod schedule;

pub use result::{Error, Result};
pub use system::{Increment, Job, System};
pub use traffic::Traffic;
pub use workload::Workload;

pub type Config = configuration::Tree;
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
