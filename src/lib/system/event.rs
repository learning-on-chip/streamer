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
