use std::cmp::Ord;

use {Config, Result, Source};
use math;
use platform::{Element, Platform};
use schedule::{Decision, Mapping, NoData, Schedule, Queue};
use system::Job;

/// A first-in-first-served scheduling policy.
pub struct Impartial {
    elements: Vec<Element>,
    queues: Vec<Queue>,
    source: Source,
}

impl Impartial {
    /// Create a scheduling policy.
    pub fn new<T: Platform>(_: &Config, platform: &T, source: Source) -> Result<Impartial> {
        let elements = platform.elements();
        Ok(Impartial {
            elements: elements.to_vec(),
            queues: elements.iter().map(|element| Queue::new(element.capacity())).collect(),
            source: source,
        })
    }
}

impl Schedule for Impartial {
    type Data = NoData;

    fn next(&mut self, job: &Job) -> Result<Decision> {
        let hosts = &self.elements;
        let guests = &job.components;
        let (have, need) = (hosts.len(), guests.len());
        let guest_order = permute(need, &mut self.source);
        let mut start = job.arrival;
        let length = job.duration();
        'outer: loop {
            let intervals = self.queues.iter().map(|queue| queue.next(start, length))
                                              .collect::<Vec<_>>();
            let host_order = sort(&intervals);
            start = intervals[host_order[0]].start();
            let mut found = vec![None; need];
            let mut taken = vec![false; have];
            'inner: for &i in &guest_order {
                for &j in &host_order {
                    if taken[j] || intervals[j].start() != start {
                        continue;
                    }
                    if guests[i].accept(&hosts[j]) {
                        found[i] = Some(j);
                        taken[j] = true;
                        continue 'inner;
                    }
                }
                for &j in &host_order[1..] {
                    if intervals[j].start() > start {
                        start = intervals[j].start();
                        continue 'outer;
                    }
                }
                raise!("failed to allocate resources for a job");
            }
            start = start.max(math::next_after(job.arrival));
            let finish = start + length;
            let mut mapping = Mapping::with_capacity(need);
            for &i in &guest_order {
                let j = some!(found[i]);
                self.queues[j].push((start, finish));
                mapping.push((i, hosts[j].id));
            }
            return Ok(Decision::accept(start, finish, mapping));
        }
    }

    fn push(&mut self, time: f64, _: Self::Data) -> Result<()> {
        for queue in &mut self.queues {
            queue.tick(time);
        }
        Ok(())
    }
}

fn permute(count: usize, source: &mut Source) -> Vec<usize> {
    use random::Source;
    use std::u64::MAX;

    if count == 0 {
        return vec![];
    }
    let scale = count as f64 / (MAX as f64 + 1.0);
    let mut order = (0..count).collect::<Vec<_>>();
    for i in 0..(count - 1) {
        order.swap(i, (scale * source.read::<u64>() as f64) as usize);
    }
    order
}

fn sort<T: Ord>(items: &[T]) -> Vec<usize> {
    let mut items = items.iter().enumerate().collect::<Vec<_>>();
    items.sort_by(|one, other| one.1.cmp(&other.1));
    items.iter().map(|item| item.0).collect()
}
