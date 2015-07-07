use std::fmt;
use std::path::Path;

use {Random, Result};
use config::Config;
use traffic::Traffic;
use workload::Workload;

pub struct System {
    pub traffic: Traffic,
    pub workload: Workload,
}

#[derive(Clone, Copy)]
pub struct State(f64);

impl System {
    pub fn new<T: AsRef<Path>>(config: T, random: &Random) -> Result<System> {
        let config = try!(Config::new(config));

        let traffic = match config.branch("traffic") {
            Some(ref traffic) => try!(Traffic::new(traffic, random)),
            _ => raise!("a traffic configuration is required"),
        };
        let workload = match config.branch("workload") {
            Some(ref workload) => try!(Workload::new(workload, random)),
            _ => raise!("a workload configuration is required"),
        };

        Ok(System {
            traffic: traffic,
            workload: workload,
        })
    }
}

impl Iterator for System {
    type Item = State;

    fn next(&mut self) -> Option<State> {
        None
    }
}

impl fmt::Display for State {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:25.15e}", self.0)
    }
}
