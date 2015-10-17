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
    stats: Stats,
    queue: BinaryHeap<Event>,
}

#[derive(Clone, Debug)]
pub struct Job {
    pub id: usize,
    pub arrival: f64,
    pub pattern: Pattern,
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
            stats: Stats::default(),
            queue: BinaryHeap::new(),
        })
    }

    getters! {
        ref platform: Platform,
        ref stats: Stats,
    }
}

impl System {
    fn tick(&mut self) -> Result<()> {
        use event::Kind::*;

        match (self.traffic.peek(), self.queue.peek()) {
            (Some(_), None) => {}
            (Some(&arrival), Some(&Event { time, .. })) if arrival < time => {},
            _ => return Ok(()),
        }

        let job = match (self.traffic.next(), self.workload.next()) {
            (Some(arrival), Some(pattern)) => {
                let id = self.stats.created;
                self.stats.created += 1;
                Job { id: id, arrival: arrival, pattern: pattern }
            },
            _ => raise!("failed to generate a new job"),
        };

        self.queue.push(Event::new(job.arrival, Arrival, job.clone()));

        let decision = try!(self.schedule.push(&job));
        try!(self.platform.push(&job, &decision));

        self.queue.push(Event::new(decision.start, Start, job.clone()));
        self.queue.push(Event::new(decision.finish, Finish, job));

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
        self.stats.account(&event);
        Some((event, power, temperature))
    }
}

impl Stats {
    fn account(&mut self, event: &Event) {
        use event::Kind::*;
        match event.kind {
            Arrival => self.arrived += 1,
            Start => self.started += 1,
            Finish => self.finished += 1,
        }
    }
}
