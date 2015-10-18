use std::fmt;

use system::Job;

/// An event.
#[derive(Clone, Debug)]
pub struct Event {
    /// The time.
    pub time: f64,
    /// The type.
    pub kind: EventKind,
    /// The job.
    pub job: Job,
}

order!(Event(time) descending);

/// The type of an event.
#[derive(Clone, Copy, Debug)]
pub enum EventKind {
    /// A job arrived.
    Arrival,
    /// A job started.
    Start,
    /// A job finished.
    Finish,
}

impl Event {
    /// Create an arrival event.
    #[inline]
    pub fn arrival(time: f64, job: Job) -> Event {
        Event { time: time, kind: EventKind::Arrival, job: job }
    }

    /// Create a start event.
    #[inline]
    pub fn start(time: f64, job: Job) -> Event {
        Event { time: time, kind: EventKind::Start, job: job }
    }

    /// Create a finish event.
    #[inline]
    pub fn finish(time: f64, job: Job) -> Event {
        Event { time: time, kind: EventKind::Finish, job: job }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let pattern = &self.job.pattern;
        write!(formatter, "{:7.2} s | job #{:3} ( {:20} | {:2} units | {:6.2} s ) {:7}",
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
