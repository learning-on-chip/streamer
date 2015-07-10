use std::collections::VecDeque;
use std::fmt;
use std::path::Path;
use std::rc::Rc;

use config::Config;
use traffic::Traffic;
use workload::{Pattern, Workload};
use {Result, Source};

pub struct System {
    time: f64,
    traffic: Traffic,
    workload: Workload,
    states: VecDeque<State>,
}

#[derive(Clone)]
pub struct State {
    pub time: f64,
    pub pattern: Rc<Pattern>,
}

impl System {
    pub fn new<T: AsRef<Path>>(config: T, source: &Source) -> Result<System> {
        let config = try!(Config::new(config));

        let traffic = match config.branch("traffic") {
            Some(ref traffic) => try!(Traffic::new(traffic, source)),
            _ => raise!("a traffic configuration is required"),
        };
        let workload = match config.branch("workload") {
            Some(ref workload) => try!(Workload::new(workload, source)),
            _ => raise!("a workload configuration is required"),
        };

        Ok(System {
            time: 0.0,
            traffic: traffic,
            workload: workload,
            states: VecDeque::new(),
        })
    }
}

impl Iterator for System {
    type Item = State;

    fn next(&mut self) -> Option<State> {
        let step = match self.traffic.next() {
            Some(step) => step,
            _ => return None,
        };
        let pattern = match self.workload.next() {
            Some(pattern) => pattern,
            _ => return None,
        };
        self.time += step;
        self.states.push_back(State {
            time: self.time,
            pattern: pattern,
        });
        self.states.pop_front()
    }
}

impl fmt::Display for State {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:10.2} s - {}", self.time, &self.pattern.name)
    }
}
