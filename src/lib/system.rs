use std::collections::BinaryHeap;
use std::fmt;
use std::path::Path;

use config::Config;
use platform::Platform;
use traffic::Traffic;
use workload::Workload;
use {Job, Result, Source};

pub struct System {
    platform: Platform,
    traffic: Traffic,
    workload: Workload,
    queue: BinaryHeap<Event>,
}

time!{
    #[derive(Clone, Debug)]
    pub struct Event {
        pub kind: EventKind,
    }
}

#[derive(Clone, Debug)]
pub enum EventKind {
    Arrival(Job),
}

impl System {
    pub fn new<T: AsRef<Path>>(config: T, source: &Source) -> Result<System> {
        let config = try!(Config::new(config));

        let platform = {
            let config = some!(config.branch("platform"), "a platform configuration is required");
            try!(Platform::new(&config))
        };
        let traffic = {
            let config = some!(config.branch("traffic"), "a traffic configuration is required");
            try!(Traffic::new(&config, source))
        };
        let workload = {
            let config = some!(config.branch("workload"), "a workload configuration is required");
            try!(Workload::new(&config, source))
        };

        Ok(System {
            platform: platform,
            traffic: traffic,
            workload: workload,
            queue: BinaryHeap::new(),
        })
    }
}

impl Iterator for System {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let time = match self.traffic.next() {
            Some(time) => time,
            _ => return None,
        };
        let job = match self.workload.next() {
            Some(pattern) => Job::new(pattern),
            _ => return None,
        };
        self.queue.push(time!(time, Event { kind: EventKind::Arrival(job.clone()) }));

        self.platform.next(job);

        self.queue.pop()
    }
}

impl fmt::Display for Event {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:9.2} s: {}", self.time, &self.kind)
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &EventKind::Arrival(ref job) => write!(formatter, "{} arrival", job),
        }
    }
}
