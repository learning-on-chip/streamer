use std::collections::BinaryHeap;
use std::fmt;

use platform::Platform;
use profile::Profile;
use traffic::Traffic;
use workload::Workload;
use {Config, Job, Result, Source};

pub struct System {
    pub platform: Platform,
    pub traffic: Traffic,
    pub workload: Workload,
    queue: BinaryHeap<Event>,
}

time! {
    #[derive(Clone, Debug)]
    pub struct Event {
        pub job: Job,
        pub kind: EventKind,
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EventKind {
    Arrival,
    Start,
    Finish,
}

pub type Increment = (Event, Profile, Profile);

impl System {
    pub fn new(config: &Config, source: &Source) -> Result<System> {
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

    #[inline]
    pub fn time_step(&self) -> f64 {
        self.platform.time_step()
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

        self.queue.push(time!(job.arrival, Event { job: job.clone(), kind: EventKind::Arrival }));

        let (start, finish) = try!(self.platform.push(&job));

        self.queue.push(time!(start, Event { job: job.clone(), kind: EventKind::Start }));
        self.queue.push(time!(finish, Event { job: job, kind: EventKind::Finish }));

        Ok(())
    }
}

impl Iterator for System {
    type Item = Increment;

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
        let pattern = &self.job.pattern;
        write!(formatter, "{:7.2} s | job #{:3} | {:15} | {:2} units | {:6.2} s | {}",
               self.time, self.job.id, pattern.name, pattern.units,
               (pattern.steps as f64) * pattern.time_step, self.kind)
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EventKind::Arrival => "arrival".fmt(formatter),
            EventKind::Start => "start".fmt(formatter),
            EventKind::Finish => "finish".fmt(formatter),
        }
    }
}
