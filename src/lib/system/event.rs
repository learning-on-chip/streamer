use schedule::Mapping;
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
    /// A job has arrived.
    Arrive(Job),
    /// A job has started.
    Start(Job, Mapping),
    /// A job has finished.
    Finish(Job, Mapping),
}

impl Event {
    /// Create a job-arrive event.
    #[inline]
    pub fn arrive(time: f64, job: Job) -> Event {
        Event { time: time, kind: EventKind::Arrive(job) }
    }

    /// Create a job-start event.
    #[inline]
    pub fn start(time: f64, job: Job, mapping: Mapping) -> Event {
        Event { time: time, kind: EventKind::Start(job, mapping) }
    }

    /// Create a job-finish event.
    #[inline]
    pub fn finish(time: f64, job: Job, mapping: Mapping) -> Event {
        Event { time: time, kind: EventKind::Finish(job, mapping) }
    }
}
