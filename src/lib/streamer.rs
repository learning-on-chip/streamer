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
extern crate time;

use std::path::Path;

#[macro_use]
mod macros;

mod event;
mod math;
mod outcome;
mod platform;
mod profile;
mod schedule;
mod system;
mod traffic;
mod workload;

pub use outcome::{Error, Result};
pub use platform::Platform;
pub use profile::Profile;
pub use system::{Increment, Job, System};

pub type Config = configuration::Tree;
pub type Source = random::Default;

pub fn open<T: AsRef<Path>>(path: T) -> Result<System> {
    let config = try!(configure(path));
    let source = {
        let seed = config.get::<i64>("seed").map(|&seed| seed as u64).unwrap_or(0);
        let seed = if seed > 0 { seed } else { time::now().to_timespec().sec as u64 };
        let seed = [0x12345678 & seed, 0x87654321 & seed];
        random::default().seed(seed)
    };
    System::new(config, source)
}

fn configure<T: AsRef<Path>>(path: T) -> Result<Config> {
    Ok(ok!(configuration::format::toml::open(path)))
}
