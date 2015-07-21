use std::cmp::Ordering;
use std::fmt;
use std::rc::Rc;

use workload::Pattern;

#[derive(Clone, Debug)]
pub struct Event {
    pub time: f64,
    pub kind: EventKind,
}

#[derive(Clone, Debug)]
pub enum EventKind {
    JobStart(Job),
}

#[derive(Clone, Debug)]
pub struct Job {
    pub id: usize,
    pub pattern: Rc<Pattern>,
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
        write!(formatter, "{:10.2} s - {}", self.time, &self.kind)
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &EventKind::JobStart(ref job) => {
                write!(formatter, "{}: {}", job.id, job.pattern.name)
            },
        }
    }
}
