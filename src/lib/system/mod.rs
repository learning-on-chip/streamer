//! Complete system.

use std::collections::BinaryHeap;

use platform::Platform;
use schedule::Schedule;
use traffic::Traffic;
use workload::Workload;
use {Outcome, Result};

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

    /// Advance to the next event and return the accumulated data.
    pub fn next(&mut self) -> Outcome<(Event, P::Data)> {
        try!(self.refill());
        let event = match self.queue.pop() {
            Some(event) => event,
            _ => return Ok(None),
        };
        self.history.remember(&event);
        try!(self.schedule.tick(event.time));
        self.platform.next(event.time).map(|data| Some((event, data)))
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

    fn refill(&mut self) -> Result<()> {
        match (try!(self.traffic.peek()), self.queue.peek()) {
            (Some(_), None) => {}
            (Some(&arrival), Some(&Event { time, .. })) if arrival < time => {},
            _ => return Ok(()),
        }

        let job = {
            let id = self.history.created;
            let arrival = some!(try!(self.traffic.next()));
            let pattern = try!(self.workload.next(arrival));
            self.history.created += 1;
            Job::new(id, arrival, pattern)
        };

        self.queue.push(Event::arrived(job.arrival, job.clone()));

        let decision = try!(self.schedule.push(&job));
        try!(self.platform.push(&job, &decision));

        self.queue.push(Event::started(decision.start, job.clone()));
        self.queue.push(Event::finished(decision.finish, job));

        Ok(())
    }
}
