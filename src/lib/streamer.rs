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

pub mod platform;
pub mod schedule;
pub mod traffic;
pub mod workload;

pub use event::Event;
pub use result::{Error, Result};
pub use system::{Job, System};

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
