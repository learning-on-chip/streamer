use std::collections::BinaryHeap;
use std::path::Path;

use config::Config;
use event::{Event, EventKind, Job};
use platform::Platform;
use traffic::Traffic;
use workload::Workload;
use {Result, Source};

pub struct System {
    time: f64,
    jobs: usize,

    platform: Platform,
    traffic: Traffic,
    workload: Workload,

    queue: BinaryHeap<Event>,
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
            time: 0.0,
            jobs: 0,

            platform: platform,
            traffic: traffic,
            workload: workload,

            queue: BinaryHeap::new(),
        })
    }
}

impl Iterator for System {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        let step = match self.traffic.next() {
            Some(step) => step,
            _ => return None,
        };
        let pattern = match self.workload.next() {
            Some(pattern) => pattern,
            _ => return None,
        };
        self.time += step;
        self.queue.push(Event {
            time: self.time,
            kind: EventKind::JobStart(Job {
                id: self.jobs,
                pattern: pattern,
            }),
        });
        self.jobs += 1;
        self.queue.pop()
    }
}
