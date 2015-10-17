use std::cmp::Ord;

use math;
use platform::Element;
use schedule::queue::Queue;
use schedule::{Decision, Schedule};
use {Job, Result};

pub struct Compact {
    elements: Vec<Element>,
    queues: Vec<Queue>,
}

impl Compact {
    pub fn new(elements: &[Element]) -> Result<Compact> {
        Ok(Compact {
            elements: elements.to_vec(),
            queues: elements.iter().map(|element| Queue::new(element.capacity())).collect(),
        })
    }
}

impl Schedule for Compact {
    fn push(&mut self, job: &Job) -> Result<Decision> {
        let pattern = &job.pattern;

        let hosts = &self.elements;
        let guests = &pattern.elements;
        let (have, need) = (hosts.len(), guests.len());

        let mut start = job.arrival;
        let length = pattern.duration();

        'outer: loop {
            let intervals = self.queues.iter().map(|queue| queue.next(start, length))
                                              .collect::<Vec<_>>();

            let order = sort(&intervals);
            start = intervals[order[0]].start();

            let mut found = vec![None; need];
            let mut taken = vec![false; have];

            'inner: for i in 0..need {
                for &j in &order {
                    if taken[j] || intervals[j].start() != start {
                        continue;
                    }
                    if guests[i].accept(&hosts[j]) {
                        found[i] = Some(j);
                        taken[j] = true;
                        continue 'inner;
                    }
                }
                for &j in &order[1..] {
                    if intervals[j].start() > start {
                        start = intervals[j].start();
                        continue 'outer;
                    }
                }
                raise!("failed to allocated resouces for a job");
            }

            start = start.max(math::next_after(job.arrival));
            let finish = start + length;
            let mut mapping = Vec::with_capacity(need);
            for i in 0..need {
                let j = some!(found[i]);
                self.queues[j].push((start, finish));
                mapping.push((i, hosts[j].id));
            }

            return Ok(Decision::new(start, finish, mapping));
        }
    }

    fn pass(&mut self, time: f64) {
        for queue in &mut self.queues {
            queue.pass(time);
        }
    }
}

fn sort<T: Ord>(items: &[T]) -> Vec<usize> {
    let mut items = items.iter().enumerate().collect::<Vec<_>>();
    items.sort_by(|one, other| one.1.cmp(&other.1));
    items.iter().map(|item| item.0).collect()
}
