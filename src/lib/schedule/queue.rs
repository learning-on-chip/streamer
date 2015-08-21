use std::collections::btree_set::{BTreeSet, Iter};
use std::f64::INFINITY;

use math;
use platform::Capacity;

pub struct Queue {
    capacity: Capacity,
    occupied: BTreeSet<Interval>,
}

#[derive(Clone, Copy, Debug)]
pub struct Interval(f64, f64);

order!(Interval(0) ascending);

struct Holes<'l> {
    from: f64,
    inner: Iter<'l, Interval>,
}

impl Queue {
    #[inline]
    pub fn new(capacity: Capacity) -> Queue {
        Queue { capacity: capacity, occupied: BTreeSet::new() }
    }

    pub fn next(&self, from: f64, length: f64) -> Interval {
        if let Capacity::Infinite = self.capacity {
             return Interval(from, INFINITY);
        }
        match self.holes(from).find(|&Interval(start, finish)| start + length <= finish) {
            Some(interval) => interval,
            _ => unreachable!(),
        }
    }

    pub fn push(&mut self, (mut start, finish): (f64, f64)) {
        debug_assert!(0.0 <= start && start <= finish);
        while self.occupied.contains(&Interval(start, 0.0)) {
            start = math::next_after(start);
        }
        self.occupied.insert(Interval(start, start.max(finish)));
    }

    pub fn pass(&mut self, time: f64) {
        let mut redundant = vec![];
        for &interval in &self.occupied {
            if interval.finish() > time {
                break;
            }
            redundant.push(interval);
        }
        for interval in redundant {
            self.occupied.remove(&interval);
        }
    }

    #[inline]
    fn holes(&self, from: f64) -> Holes {
        Holes { from: from, inner: self.occupied.iter() }
    }
}

impl<'l> Iterator for Holes<'l> {
    type Item = Interval;

    fn next(&mut self) -> Option<Self::Item> {
        let from = self.from;
        if from.is_infinite() {
            return None;
        }
        match self.inner.next() {
            Some(&Interval(start, finish)) => {
                if from < start {
                    self.from = finish;
                    return Some(Interval(from, start));
                }
                if from < finish {
                    self.from = finish;
                }
                return self.next();
            },
            _ => {
                self.from = INFINITY;
                return Some(Interval(from, INFINITY));
            },
        }
    }
}

impl Interval {
    #[inline(always)]
    pub fn start(&self) -> f64 {
        self.0
    }

    #[inline(always)]
    pub fn finish(&self) -> f64 {
        self.1
    }
}

#[cfg(test)]
mod tests {
    use platform::Capacity;
    use schedule::queue::{Interval, Queue};
    use std::f64::INFINITY;

    macro_rules! test(
        ($queue:ident, $from:expr, [$(($start:expr, $finish:expr)),+]) => ({
            let intervals = $queue.holes($from).collect::<Vec<_>>();
            assert_eq!(intervals, vec![$(Interval($start, $finish)),+]);
        });
    );

    #[test]
    fn push() {
        let mut queue = Queue::new(Capacity::Single);

        test!(queue, 0.0, [(0.0, INFINITY)]);
        test!(queue, 10.0, [(10.0, INFINITY)]);

        queue.push((5.0, 15.0));
        test!(queue, 10.0, [(15.0, INFINITY)]);

        queue.push((15.0, 20.0));
        test!(queue, 10.0, [(20.0, INFINITY)]);
        test!(queue, 15.0, [(20.0, INFINITY)]);
        test!(queue, 20.0, [(20.0, INFINITY)]);

        queue.push((16.0, 42.0));
        test!(queue, 0.0, [(0.0, 5.0), (42.0, INFINITY)]);
    }

    #[test]
    fn push_duplicate() {
        let mut queue = Queue::new(Capacity::Single);

        queue.push((1.0, 2.0));
        queue.push((1.0, 4.0));

        assert_eq!(queue.occupied.len(), 2);
        test!(queue, 0.0, [(0.0, 1.0), (4.0, INFINITY)]);
    }

    #[test]
    fn pass() {
        let mut queue = Queue::new(Capacity::Single);

        queue.push((10.0, 15.0));
        queue.push((15.0, 20.0));
        queue.push((25.0, 30.0));

        queue.pass(10.0);
        test!(queue, 0.0, [(0.0, 10.0), (20.0, 25.0), (30.0, INFINITY)]);

        queue.pass(11.0);
        test!(queue, 0.0, [(0.0, 10.0), (20.0, 25.0), (30.0, INFINITY)]);

        queue.pass(15.0);
        test!(queue, 0.0, [(0.0, 15.0), (20.0, 25.0), (30.0, INFINITY)]);

        queue.pass(20.0);
        test!(queue, 0.0, [(0.0, 25.0), (30.0, INFINITY)]);

        queue.pass(30.0);
        test!(queue, 0.0, [(0.0, INFINITY)]);
    }
}
