use std::cmp::Ordering;
use std::fmt;

use Job;

#[derive(Clone, Debug)]
pub struct Event {
    pub time: f64,
    pub kind: EventKind,
}

#[derive(Clone, Debug)]
pub enum EventKind {
    JobArrival(Job),
    JobStart(Job),
    JobFinish(Job),
}

impl Eq for Event {
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.time < other.time {
            Ordering::Greater
        } else if self.time > other.time {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl PartialEq for Event {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl PartialOrd for Event {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Event {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:9.2} s: {}", self.time, &self.kind)
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &EventKind::JobArrival(ref job) => write!(formatter, "{} arrival", job),
            &EventKind::JobStart(ref job) => write!(formatter, "{} start", job),
            &EventKind::JobFinish(ref job) => write!(formatter, "{} finish", job),
        }
    }
}
