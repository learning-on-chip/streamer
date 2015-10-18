use std::collections::BinaryHeap;

use Result;
use platform::Platform;
use schedule::Schedule;
use traffic::Traffic;
use workload::Workload;

mod event;
mod job;

pub use self::event::{Event, EventKind};
pub use self::job::Job;

pub struct System<P, S, T, W> where P: Platform, S: Schedule, T: Traffic, W: Workload {
    platform: P,
    schedule: S,
    traffic: T,
    workload: W,
    queue: BinaryHeap<Event>,
    statistics: Statistics,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Statistics {
    pub created: usize,
    pub arrived: usize,
    pub started: usize,
    pub finished: usize,
}

impl<P, S, T, W> System<P, S, T, W> where P: Platform, S: Schedule, T: Traffic, W: Workload {
    pub fn new(platform: P, schedule: S, traffic: T, workload: W) -> Result<System<P, S, T, W>> {
        Ok(System {
            platform: platform,
            schedule: schedule,
            traffic: traffic,
            workload: workload,
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

        let job = {
            let id = self.statistics.created;
            let arrival = some!(self.traffic.next(), "failed to generate an arrival");
            let pattern = some!(self.workload.next(arrival), "failed to generate a workload");
            self.statistics.created += 1;
            Job::new(id, arrival, pattern)
        };

        self.queue.push(Event::arrival(job.arrival, job.clone()));

        let decision = try!(self.schedule.push(&job));
        try!(self.platform.push(&job, &decision));

        self.queue.push(Event::start(decision.start, job.clone()));
        self.queue.push(Event::finish(decision.finish, job));

        Ok(())
    }

    getters! {
        ref platform: P,
        ref statistics: Statistics,
    }
}

impl<P, S, T, W> Iterator for System<P, S, T, W>
    where P: Platform, S: Schedule, T: Traffic, W: Workload
{
    type Item = (Event, P::Output);

    fn next(&mut self) -> Option<Self::Item> {
        if let Err(error ) = self.tick() {
            error!(target: "System", "Failed to update the state ({}).", error);
            return None;
        }
        let event = match self.queue.pop() {
            Some(event) => event,
            _ => return None,
        };
        self.schedule.tick(event.time);
        self.statistics.account(&event);
        self.platform.next(event.time).map(|output| (event, output))
    }
}

impl Statistics {
    fn account(&mut self, event: &Event) {
        match event.kind {
            EventKind::Arrival => self.arrived += 1,
            EventKind::Start => self.started += 1,
            EventKind::Finish => self.finished += 1,
        }
    }
}
