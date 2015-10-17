use std::fmt;

use system::Job;

#[derive(Clone, Debug)]
pub struct Event {
    pub time: f64,
    pub kind: Kind,
    pub job: Job,
}

order!(Event(time) descending);

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Arrival,
    Start,
    Finish,
}

impl Event {
    #[inline]
    pub fn arrival(time: f64, job: Job) -> Event {
        Event { time: time, kind: Kind::Arrival, job: job }
    }

    #[inline]
    pub fn start(time: f64, job: Job) -> Event {
        Event { time: time, kind: Kind::Start, job: job }
    }

    #[inline]
    pub fn finish(time: f64, job: Job) -> Event {
        Event { time: time, kind: Kind::Finish, job: job }
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

impl fmt::Display for Kind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Kind::Arrival => "arrival".fmt(formatter),
            Kind::Start => "start".fmt(formatter),
            Kind::Finish => "finish".fmt(formatter),
        }
    }
}
