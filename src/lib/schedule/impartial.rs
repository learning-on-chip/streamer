use std::cmp::Ord;

use math;
use platform::{Element, Platform};
use schedule::{Decision, NoData, Schedule, Queue};
use system::Job;
use {Config, Result};

/// A first-in-first-served scheduling policy.
pub struct Impartial {
    elements: Vec<Element>,
    queues: Vec<Queue>,
    pending: Vec<f64>,
    capacity: usize,
}

impl Impartial {
    /// Create a scheduling policy.
    pub fn new<T: Platform>(config: &Config, platform: &T) -> Result<Impartial> {
        let elements = platform.elements();
        let queues = elements.iter().map(|element| Queue::new(element.capacity())).collect();
        let capacity = config.get::<i64>("capacity").map(|value| *value as usize)
                                                    .unwrap_or(usize::max_value());
        Ok(Impartial {
            elements: elements.to_vec(),
            queues: queues,
            pending: vec![],
            capacity: capacity,
        })
    }
}

impl Schedule for Impartial {
    type Data = NoData;

    fn next(&mut self, job: &Job) -> Result<Decision> {
        if self.pending.len() == self.capacity {
            return Ok(Decision::reject());
        }

        let hosts = &self.elements;
        let guests = &job.elements;
        let (have, need) = (hosts.len(), guests.len());

        let mut start = job.arrival;
        let length = job.duration();

        let mut found = vec![None; need];
        let mut taken = vec![false; have];

        'outer: loop {
            let intervals = self.queues.iter().map(|queue| queue.next(start, length))
                                              .collect::<Vec<_>>();

            let order = sort(&intervals);
            start = intervals[order[0]].start();

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
                        for item in &mut found {
                            *item = None;
                        }
                        for item in &mut taken {
                            *item = false;
                        }
                        continue 'outer;
                    }
                }
                raise!("failed to allocated resources for a job");
            }

            break;
        }

        start = start.max(math::next_after(job.arrival));
        self.pending.push(start);

        let finish = start + length;
        let mut mapping = Vec::with_capacity(need);
        for i in 0..need {
            let j = some!(found[i]);
            self.queues[j].push((start, finish));
            mapping.push((i, hosts[j].id));
        }

        return Ok(Decision::accept(start, finish, mapping));
    }

    fn push(&mut self, time: f64, _: Self::Data) -> Result<()> {
        self.pending.retain(|&start| start > time);
        for queue in &mut self.queues {
            queue.tick(time);
        }
        Ok(())
    }
}

fn sort<T: Ord>(items: &[T]) -> Vec<usize> {
    let mut items = items.iter().enumerate().collect::<Vec<_>>();
    items.sort_by(|one, other| one.1.cmp(&other.1));
    items.iter().map(|item| item.0).collect()
}
