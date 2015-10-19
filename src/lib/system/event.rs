use std::fmt;

use system::Job;

/// An event.
#[derive(Clone, Debug)]
pub struct Event {
    /// The time.
    pub time: f64,
    /// The type.
    pub kind: EventKind,
}

order!(Event(time) descending);

/// The type of an event.
#[derive(Clone, Debug)]
pub enum EventKind {
    /// A job arrived.
    Arrived(Job),
    /// A job started.
    Started(Job),
    /// A job finished.
    Finished(Job),
}

impl Event {
    /// Create a job-arrived event.
    #[inline]
    pub fn arrived(time: f64, job: Job) -> Event {
        Event { time: time, kind: EventKind::Arrived(job) }
    }

    /// Create a job-started event.
    #[inline]
    pub fn started(time: f64, job: Job) -> Event {
        Event { time: time, kind: EventKind::Started(job) }
    }

    /// Create a job-finished event.
    #[inline]
    pub fn finished(time: f64, job: Job) -> Event {
        Event { time: time, kind: EventKind::Finished(job) }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let (job, kind) = match &self.kind {
            &EventKind::Arrived(ref job) => (job, "arrival"),
            &EventKind::Started(ref job) => (job, "start"),
            &EventKind::Finished(ref job) => (job, "finish"),
        };
        let pattern = &job.pattern;
        write!(formatter, "{:7.2} s | job #{:3} ( {:20} | {:2} units | {:6.2} s ) {:7}",
               self.time, job.id, pattern.name, pattern.units,
               (pattern.steps as f64) * pattern.time_step, kind)
    }
}
