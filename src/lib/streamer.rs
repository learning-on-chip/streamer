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
mod math;
mod platform;
mod profile;
mod result;
mod schedule;
mod system;
mod traffic;
mod workload;

pub use platform::Platform;
pub use profile::Profile;
pub use result::{Error, Result};
pub use system::{Increment, Job, System};

pub type Config = configuration::Tree;
pub type Source = random::Default;
