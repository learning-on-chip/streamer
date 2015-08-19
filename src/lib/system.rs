use std::collections::BinaryHeap;
use std::fmt;

use platform::Platform;
use profile::Profile;
use traffic::Traffic;
use workload::{Pattern, Workload};
use {Config, Result, Source};

pub struct System {
    pub platform: Platform,
    pub traffic: Traffic,
    pub workload: Workload,
    pub stats: Stats,
    queue: BinaryHeap<Event>,
}

#[derive(Clone, Debug)]
pub struct Job {
    pub id: usize,
    pub arrival: f64,
    pub pattern: Pattern,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub time: f64,
    pub job: Job,
    pub kind: EventKind,
}

time!(Event);

#[derive(Clone, Copy, Debug)]
pub enum EventKind {
    Arrival,
    Start,
    Finish,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub created: usize,
    pub arrived: usize,
    pub started: usize,
    pub finished: usize,
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
            stats: Stats::default(),
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

    fn create_job(&mut self, arrival: f64, pattern: Pattern) -> Job {
        let id = self.stats.created;
        self.stats.created += 1;
        Job { id: id, arrival: arrival, pattern: pattern }
    }

    fn enqueue_job(&mut self) -> Result<()> {
        let job = match (self.traffic.next(), self.workload.next()) {
            (Some(arrival), Some(pattern)) => self.create_job(arrival, pattern),
            _ => raise!("failed to generate a new job"),
        };

        self.queue.push(Event { time: job.arrival, job: job.clone(), kind: EventKind::Arrival });

        let (start, finish) = try!(self.platform.push(&job));

        self.queue.push(Event { time: start, job: job.clone(), kind: EventKind::Start });
        self.queue.push(Event { time: finish, job: job, kind: EventKind::Finish });

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
        self.stats.account(&event);
        Some((event, power, temperature))
    }
}

impl fmt::Display for Event {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let pattern = &self.job.pattern;
        write!(formatter, "{:7.2} s | job #{:3} | {:20} | {:2} units | {:6.2} s | {:7}",
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

impl Stats {
    fn account(&mut self, event: &Event) {
        match event.kind {
            EventKind::Arrival => self.arrived += 1,
            EventKind::Start => self.started += 1,
            EventKind::Finish => self.finished += 1,
        }
    }
}
