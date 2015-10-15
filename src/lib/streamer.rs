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

mod platform;
mod profile;
mod schedule;
mod system;
mod traffic;
mod workload;

pub use platform::Platform;
pub use profile::Profile;
pub use system::{Increment, Job, System};

pub struct Error(String);
pub type Result<T> = std::result::Result<T, Error>;

pub type Config = configuration::Tree;
pub type Source = random::Default;

impl Error {
    #[inline]
    pub fn new<T: ToString>(message: T) -> Error {
        Error(message.to_string())
    }
}

impl fmt::Debug for Error {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl error::Error for Error {
    #[inline]
    fn description(&self) -> &str {
        &self.0
    }
}

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
    let path = path.as_ref();
    let mut config = ok!(configuration::format::toml::open(path));
    if let Some(root) = path.parent() {
        if config.set("root", root.to_path_buf()).is_none() {
            raise!("failed to set the root directory");
        }
    }
    Ok(config)
}
