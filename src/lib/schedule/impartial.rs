use std::cmp::Ord;
use std::marker::PhantomData;

use math;
use platform::Element;
use schedule::{Decision, Schedule, Queue};
use system::Job;
use {Config, Result};

/// A first-in-first-served scheduling policy.
pub struct Impartial<T> {
    elements: Vec<Element>,
    queues: Vec<Queue>,
    phantom: PhantomData<T>,
}

impl<T> Impartial<T> {
    /// Create a scheduling policy.
    pub fn new(_: &Config, elements: &[Element]) -> Result<Impartial<T>> {
        Ok(Impartial {
            elements: elements.to_vec(),
            queues: elements.iter().map(|element| Queue::new(element.capacity())).collect(),
            phantom: PhantomData,
        })
    }
}

impl<T> Schedule for Impartial<T> {
    type Data = T;

    fn push(&mut self, job: &Job) -> Result<Decision> {
        let hosts = &self.elements;
        let guests = &job.elements;
        let (have, need) = (hosts.len(), guests.len());

        let mut start = job.arrival;
        let length = job.duration();

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
                raise!("failed to allocated resources for a job");
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

    fn step(&mut self, time: f64, _: &Self::Data) -> Result<()> {
        for queue in &mut self.queues {
            queue.step(time);
        }
        Ok(())
    }
}

fn sort<T: Ord>(items: &[T]) -> Vec<usize> {
    let mut items = items.iter().enumerate().collect::<Vec<_>>();
    items.sort_by(|one, other| one.1.cmp(&other.1));
    items.iter().map(|item| item.0).collect()
}
