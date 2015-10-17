use std::collections::BinaryHeap;

use Result;
use event::Event;
use platform::{Platform, Profile};
use schedule::Schedule;
use traffic::Traffic;
use workload::{Pattern, Workload};

pub struct System {
    platform: Platform,
    schedule: Box<Schedule>,
    traffic: Box<Traffic>,
    workload: Box<Workload>,
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
    pub fn new<S, T, W>(platform: Platform, schedule: S, traffic: T, workload: W)
                        -> Result<System>
        where S: 'static + Schedule, T: 'static + Traffic, W: 'static + Workload
    {
        Ok(System {
            platform: platform,
            schedule: Box::new(schedule),
            traffic: Box::new(traffic),
            workload: Box::new(workload),
            queue: BinaryHeap::new(),
            statistics: Statistics::default(),
        })
    }

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

    getters! {
        ref platform: Platform,
        ref statistics: Statistics,
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
