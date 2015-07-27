use std::collections::BinaryHeap;
use std::fmt;
use std::path::Path;

use config::Config;
use platform::Platform;
use profile::Profile;
use traffic::Traffic;
use workload::Workload;
use {Job, Result, Source};

pub struct System {
    platform: Platform,
    traffic: Traffic,
    workload: Workload,
    queue: BinaryHeap<Event>,
}

time! {
    #[derive(Clone, Debug)]
    pub struct Event {
        pub kind: EventKind,
    }
}

#[derive(Clone, Debug)]
pub enum EventKind {
    Arrival(Job),
    Start(Job),
    Finish(Job),
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

    fn update(&mut self) -> Result<()> {
        match (self.traffic.peek(), self.queue.peek()) {
            (Some(_), None) => try!(self.enqueue_job()),
            (Some(&arrival), Some(&Event { time, .. })) => if arrival < time {
                try!(self.enqueue_job());
            },
            _ => {},
        }
        Ok(())
    }

    fn enqueue_job(&mut self) -> Result<()> {
        let job = match (self.traffic.next(), self.workload.next()) {
            (Some(arrival), Some(pattern)) => Job::new(arrival, pattern),
            _ => raise!("failed to generate a new job"),
        };

        self.queue.push(time!(job.arrival, Event {
            kind: EventKind::Arrival(job.clone()),
        }));

        let (start, finish) = try!(self.platform.push(&job));

        self.queue.push(time!(start, Event {
            kind: EventKind::Start(job.clone()),
        }));
        self.queue.push(time!(finish, Event {
            kind: EventKind::Finish(job),
        }));

        Ok(())
    }
}

impl Iterator for System {
    type Item = (Event, Profile, Profile);

    fn next(&mut self) -> Option<Self::Item> {
        if let Err(error) = self.update() {
            error!(target: "System", "Failed to update the state ({}).", error);
        }
        let event = match self.queue.pop() {
            Some(event) => event,
            _ => return None,
        };
        let (power, temperature) = match self.platform.next(event.time) {
            Some((power, temperature)) => (power, temperature),
            _ => return None,
        };
        Some((event, power, temperature))
    }
}

impl fmt::Display for Event {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:9.2} s: {:<30}", self.time, &self.kind)
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        macro_rules! job(
            ($job:expr, $note:expr) => (
                write!(formatter, "{} {}", $job, $note)
            )
        );

        match self {
            &EventKind::Arrival(ref job) => job!(job, "arrival"),
            &EventKind::Start(ref job) => job!(job, "start"),
            &EventKind::Finish(ref job) => job!(job, "finish"),
        }
    }
}
