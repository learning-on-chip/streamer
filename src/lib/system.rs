use std::collections::BinaryHeap;

use event::Event;
use platform::Platform;
use profile::Profile;
use schedule::{self, Schedule};
use traffic::Traffic;
use workload::{Pattern, Workload};
use {Config, Result, Source};

pub struct System {
    platform: Platform,
    schedule: Box<Schedule>,
    traffic: Traffic,
    workload: Workload,
    queue: BinaryHeap<Event>,
    statistics: Statistics,
}

#[derive(Clone, Debug)]
pub struct Job {
    pub id: usize,
    pub arrival: f64,
    pub pattern: Pattern,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Statistics {
    pub created: usize,
    pub arrived: usize,
    pub started: usize,
    pub finished: usize,
}

pub type Increment = (Event, Profile, Profile);

impl System {
    pub fn new(config: Config, source: Source) -> Result<System> {
        let platform = {
            let config = some!(config.branch("platform"), "a platform configuration is required");
            try!(Platform::new(&config))
        };
        let schedule = {
            Box::new(try!(schedule::Compact::new(platform.elements())))
        };
        let traffic = {
            let config = some!(config.branch("traffic"), "a traffic configuration is required");
            try!(Traffic::new(&config, &source))
        };
        let workload = {
            let config = some!(config.branch("workload"), "a workload configuration is required");
            try!(Workload::new(&config, &source))
        };

        Ok(System {
            platform: platform,
            schedule: schedule,
            traffic: traffic,
            workload: workload,
            queue: BinaryHeap::new(),
            statistics: Statistics::default(),
        })
    }

    getters! {
        ref platform: Platform,
        ref statistics: Statistics,
    }
}

impl System {
    fn tick(&mut self) -> Result<()> {
        match (self.traffic.peek(), self.queue.peek()) {
            (Some(_), None) => {}
            (Some(&arrival), Some(&Event { time, .. })) if arrival < time => {},
            _ => return Ok(()),
        }

        let job = match (self.traffic.next(), self.workload.next()) {
            (Some(arrival), Some(pattern)) => {
                let id = self.statistics.created;
                self.statistics.created += 1;
                Job::new(id, arrival, pattern)
            },
            _ => raise!("failed to generate a new job"),
        };

        self.queue.push(Event::arrival(job.arrival, job.clone()));

        let decision = try!(self.schedule.push(&job));
        try!(self.platform.push(&job, &decision));

        self.queue.push(Event::start(decision.start, job.clone()));
        self.queue.push(Event::finish(decision.finish, job));

        Ok(())
    }
}

impl Iterator for System {
    type Item = Increment;

    fn next(&mut self) -> Option<Self::Item> {
        if let Err(error) = self.tick() {
            error!(target: "System", "Failed to update the state ({}).", error);
        }
        let event = match self.queue.pop() {
            Some(event) => event,
            _ => return None,
        };
        self.schedule.pass(event.time);
        let (power, temperature) = match self.platform.next(event.time) {
            Some((power, temperature)) => (power, temperature),
            _ => return None,
        };
        self.statistics.account(&event);
        Some((event, power, temperature))
    }
}

impl Job {
    #[inline]
    pub fn new(id: usize, arrival: f64, pattern: Pattern) -> Job {
        Job { id: id, arrival: arrival, pattern: pattern }
    }
}

impl Statistics {
    fn account(&mut self, event: &Event) {
        use event::Kind::*;
        match event.kind {
            Arrival => self.arrived += 1,
            Start => self.started += 1,
            Finish => self.finished += 1,
        }
    }
}
