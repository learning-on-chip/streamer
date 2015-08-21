use std::collections::btree_map::{BTreeMap, Values};
use std::f64::INFINITY;
use std::ops::Deref;

pub struct Queue {
    occupied: BTreeMap<Start, Interval>,
}

pub struct Vacancies<'l> {
    from: f64,
    last: Option<Interval>,
    inner: Values<'l, Start, Interval>,
}

#[derive(Clone, Copy)]
struct Start(f64);

order!(Start(0) ascending);

#[derive(Clone, Copy, Debug)]
pub struct Interval(pub f64, pub f64);

order!(Interval(0) ascending);

impl Queue {
    #[inline]
    pub fn new() -> Queue {
        Queue { occupied: BTreeMap::new() }
    }

    pub fn push(&mut self, (mut start, finish): (f64, f64)) {
        while self.occupied.contains_key(&Start(start)) {
            start = unsafe { m::nextafter(start, INFINITY) };
        }
        self.occupied.insert(Start(start), Interval(start, finish));
    }

    #[inline]
    pub fn vacancies(&self, from: f64) -> Vacancies {
        Vacancies { from: from, last: None, inner: self.occupied.values() }
    }
}

impl<'l> Vacancies<'l> {
    #[inline]
    fn last(&self) -> Option<Self::Item> {
        self.last
    }
}

impl<'l> Iterator for Vacancies<'l> {
    type Item = Interval;

    fn next(&mut self) -> Option<Self::Item> {
        let from = self.from;
        if from.is_infinite() {
            self.last = None;
            return self.last;
        }
        match self.inner.next() {
            Some(&(start, finish)) => {
                if from < start {
                    self.from = finish;
                    self.last = Some(Interval(from, start));
                    return self.last;
                }
                if from < finish {
                    self.from = finish;
                }
                return self.next();
            },
            _ => {
                self.from = INFINITY;
                self.last = Some(Interval(from, INFINITY));
                return self.last;
            },
        }
    }
}

impl Interval {
    #[inline]
    pub fn allows(&self, start: f64, length: f64) -> bool {
        if start <= self.0 { self.0 + length <= self.1 } else { start + length <= self.1 }
    }
}

mod m {
    #[link_name = "m"]
    extern {
        pub fn nextafter(x: f64, y: f64) -> f64;
    }
}

#[cfg(test)]
mod tests {
    use std::f64::INFINITY;
    use super::{Interval, Queue};

    macro_rules! test(
        ($queue:ident, $from:expr, [$(($start:expr, $finish:expr)),+]) => ({
            let intervals = $queue.vacancies($from).collect::<Vec<_>>();
            assert_eq!(intervals, vec![$(Interval($start, $finish)),+]);
        });
    );

    #[test]
    fn vacancies() {
        let mut queue = Queue::new();

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
        let mut queue = Queue::new();

        queue.push((1.0, 2.0));
        queue.push((1.0, 4.0));

        assert_eq!(queue.occupied.len(), 2);
        test!(queue, 0.0, [(0.0, 1.0), (4.0, INFINITY)]);
    }
}
