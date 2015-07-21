use std::collections::BinaryHeap;
use std::path::Path;

use config::Config;
use event::{Event, EventKind, Job};
use id::ID;
use platform::Platform;
use traffic::Traffic;
use workload::Workload;
use {Result, Source};

pub struct System {
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
        let time = match self.traffic.next() {
            Some(time) => time,
            _ => return None,
        };
        let pattern = match self.workload.next() {
            Some(pattern) => pattern,
            _ => return None,
        };
        self.queue.push(Event {
            time: time,
            kind: EventKind::JobArrival(Job {
                id: ID::new("job"),
                pattern: pattern,
            }),
        });

        self.queue.pop()
    }
}
