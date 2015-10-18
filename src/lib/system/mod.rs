//! Complete system.

use std::collections::BinaryHeap;

use Result;
use platform::Platform;
use schedule::Schedule;
use traffic::Traffic;
use workload::Workload;

mod event;
mod history;
mod job;

pub use self::event::{Event, EventKind};
pub use self::history::History;
pub use self::job::Job;

/// A complete system.
pub struct System<T, W, P, S> where T: Traffic, W: Workload, P: Platform, S: Schedule {
    traffic: T,
    workload: W,
    platform: P,
    schedule: S,
    history: History,
    queue: BinaryHeap<Event>,
}

impl<T, W, P, S> System<T, W, P, S> where T: Traffic, W: Workload, P: Platform, S: Schedule {
    /// Create a system.
    pub fn new(traffic: T, workload: W, platform: P, schedule: S) -> Result<System<T, W, P, S>> {
        Ok(System {
            traffic: traffic,
            workload: workload,
            platform: platform,
            schedule: schedule,
            history: History::default(),
            queue: BinaryHeap::new(),
        })
    }

    /// Return the platform.
    #[inline(always)]
    pub fn platform(&self) -> &P {
        &self.platform
    }

    /// Return the history.
    #[inline(always)]
    pub fn history(&self) -> &History {
        &self.history
    }

    fn tick(&mut self) -> Result<()> {
        match (self.traffic.peek(), self.queue.peek()) {
            (Some(_), None) => {}
            (Some(&arrival), Some(&Event { time, .. })) if arrival < time => {},
            _ => return Ok(()),
        }

        let job = {
            let id = self.history.created;
            let arrival = some!(self.traffic.next(), "failed to generate an arrival");
            let pattern = some!(self.workload.next(arrival), "failed to generate a workload");
            self.history.created += 1;
            Job::new(id, arrival, pattern)
        };

        self.queue.push(Event::arrival(job.arrival, job.clone()));

        let decision = try!(self.schedule.push(&job));
        try!(self.platform.push(&job, &decision));

        self.queue.push(Event::start(decision.start, job.clone()));
        self.queue.push(Event::finish(decision.finish, job));

        Ok(())
    }
}

impl<T, W, P, S> Iterator for System<T, W, P, S>
    where T: Traffic, W: Workload, P: Platform, S: Schedule
{
    type Item = (Event, P::Data);

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
        self.history.remember(&event);
        self.platform.next(event.time).map(|data| (event, data))
    }
}
