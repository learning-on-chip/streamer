//! System modeling.

use std::collections::BinaryHeap;

use Result;
use platform::Platform;
use schedule::{Decision, Schedule};
use traffic::Traffic;
use workload::Workload;

mod event;
mod history;
mod job;

pub use self::event::{Event, EventKind};
pub use self::history::History;
pub use self::job::Job;

/// A system.
pub struct System<T, W, P, S> {
    traffic: T,
    workload: W,
    platform: P,
    schedule: S,
    history: History,
    queue: BinaryHeap<Event>,
}

impl<T, W, P, S> System<T, W, P, S>
    where T: Traffic, W: Workload, P: Platform, S: Schedule, S::Data: for<'l> From<&'l P::Data>
{
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

    /// Advance to the next event and return the data accumulated since the
    /// previous call.
    pub fn next(&mut self) -> Result<Option<(Event, P::Data)>> {
        match (try!(self.traffic.peek()), self.queue.peek().map(|event| &event.time)) {
            (Some(&traffic), Some(&queue)) => if traffic < queue {
                self.next_from_traffic()
            } else {
                self.next_from_queue()
            },
            (Some(_), None) => self.next_from_traffic(),
            (None, Some(_)) => self.next_from_queue(),
            _ => Ok(None),
        }
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

    fn next_from_traffic(&mut self) -> Result<Option<(Event, P::Data)>> {
        let time = some!(try!(self.traffic.next()));
        let pattern = try!(self.workload.next(time));
        let job = Job::new(self.history.arrived, time, pattern);

        let event = Event::arrived(time, job.clone());
        self.history.count(&event);

        let data = try!(self.platform.next(time));
        try!(self.schedule.push(time, (&data).into()));

        if let Decision::Accept { start, finish, mapping } = try!(self.schedule.next(&job)) {
            self.queue.push(Event::started(start, job.clone()));
            self.queue.push(Event::finished(finish, job.clone()));
            try!(self.platform.push(&job, start, &mapping));
        }

        Ok(Some((event, data)))
    }

    fn next_from_queue(&mut self) -> Result<Option<(Event, P::Data)>> {
        let event = some!(self.queue.pop());
        self.history.count(&event);

        let data = try!(self.platform.next(event.time));
        try!(self.schedule.push(event.time, (&data).into()));

        Ok(Some((event, data)))
    }
}
