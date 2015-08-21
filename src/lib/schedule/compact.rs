use std::collections::BinaryHeap;

use platform::{Element, ElementKind};
use schedule::{Interval, Schedule, Queue};
use {Job, Result};

pub struct Compact {
    elements: Vec<Element>,
    queues: Vec<Queue>,
}

impl Compact {
    pub fn new(elements: &[Element]) -> Result<Compact> {
        Ok(Compact { elements: elements.clone(), queues: vec![Queue::new(); elements.len()] })
    }
}

impl Schedule for Compact {
    fn push(&mut self, job: &Job) -> Result<(f64, f64, Vec<(usize, usize)>)> {
        use std::f64::EPSILON;

        let pattern = &job.pattern;

        let (have, need) = (self.elements.len(), pattern.elements.len());
        if have < need {
            raise!("do not have enough resources for a job");
        }

        let mut start = job.arrival;
        let duration = pattern.duration();

        let mut intervals = vec![Interval(0.0, 0.0); have];
        let mut vacancies = self.queues.iter().map(|queue| queue.vacancies(start))
                                              .collect::<Vec<_>>();

        loop {
            for i in 0..have {
                while intervals[i].length() < duration {
                    intervals[i] = match vacancies.next() {
                        Some(interval) => interval,
                        _ => raise!("failed to find a long enough time interval"),
                    }
                }
            }
            let mut found = vec![None; need];
            for i in 0..need {
                let requested = &pattern.elements[i];
                for j in 0..have {
                    let candidate = &self.elements[j];
                    if candidate.kind != requested.kind {
                        continue;
                    }
                    if candidate.shared() {
                        found[i] = Some(j);
                        break;
                    }
                }
            }
            if found.iter().all(|&found| found) {
                break;
            }
        }

        let start = job.arrival.max(available) + EPSILON;
        let finish = start + duration;
        let mut mapping = Vec::with_capacity(units);
        for i in 0..units {
            mapping.push((i, hosts[i].element.id));
        }

        Ok((start, finish, mapping))
    }
}
