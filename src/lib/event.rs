use std::fmt;

use system::Job;

#[derive(Clone, Debug)]
pub struct Event {
    pub time: f64,
    pub kind: EventKind,
    pub job: Job,
}

order!(Event(time) descending);

#[derive(Clone, Copy, Debug)]
pub enum EventKind {
    Arrival,
    Start,
    Finish,
}

impl Event {
    #[inline]
    pub fn new(time: f64, kind: EventKind, job: Job) -> Event {
        Event { time: time, kind: kind, job: job }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let pattern = self.job.pattern();
        write!(formatter, "{:7.2} s | job #{:3} ( {:20} | {:2} units | {:6.2} s ) {:7}",
               self.time, self.job.id(), pattern.name, pattern.units,
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
