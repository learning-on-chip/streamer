use std::collections::{BinaryHeap, HashMap};
use std::ops::Deref;

use platform::{Element, ElementKind};
use schedule::Schedule;
use {Job, Result};

pub struct Compact {
    units: usize,
    hosts: HashMap<ElementKind, BinaryHeap<Host>>,
}

pub struct Host {
    time: f64,
    element: Element,
}

order!(Host(time) descending);

impl Compact {
    pub fn new(elements: &[Element]) -> Result<Compact> {
        let mut hosts = HashMap::new();
        for element in elements {
            let heap = hosts.entry(element.kind).or_insert_with(|| BinaryHeap::new());
            heap.push(Host { time: 0.0, element: element.clone() });
        }
        Ok(Compact { units: elements.len(), hosts: hosts })
    }
}

impl Schedule for Compact {
    fn push(&mut self, job: &Job) -> Result<(f64, f64, Vec<(usize, usize)>)> {
        use std::f64::EPSILON;

        let pattern = &job.pattern;

        let units = pattern.units;
        if self.units < units {
            raise!("do not have enough resources for a job");
        }

        let mut available = 0f64;
        let mut hosts = vec![];
        for element in &pattern.elements {
            match self.hosts.get_mut(&element.kind).and_then(|heap| heap.pop()) {
                Some(host) => {
                    if host.is_exclusive() {
                        available = available.max(host.time);
                    }
                    hosts.push(host);
                },
                _ => break,
            }
        }

        macro_rules! commit(
            () => (for host in hosts.drain(..) {
                self.hosts.get_mut(&host.kind).unwrap().push(host);
            });
        );

        if hosts.len() != units {
            commit!();
            raise!("failed to allocate resources for a job");
        }

        let start = job.arrival.max(available) + EPSILON;
        let finish = start + pattern.duration();
        let mut mapping = Vec::with_capacity(units);
        for i in 0..units {
            mapping.push((i, hosts[i].element.id));
        }

        for host in &mut hosts {
            host.time = finish;
        }
        commit!();

        Ok((start, finish, mapping))
    }
}

impl Deref for Host {
    type Target = Element;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.element
    }
}
