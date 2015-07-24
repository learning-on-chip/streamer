use std::collections::{BinaryHeap, HashMap};
use std::str::FromStr;
use temperature::circuit::ThreeDICE;
use temperature::{self, Simulator};
use threed_ice::{StackElement, System};

use config::Config;
use profile::Profile;
use {Error, ID, Job, Result};

pub struct Platform {
    pub units: usize,
    pub elements: HashMap<ElementKind, BinaryHeap<Element>>,
    pub simulator: Simulator,
    pub power: Profile,
    pub temperature: Profile,
}

time! {
    #[derive(Clone, Copy, Debug)]
    pub struct Element {
        pub id: ID,
        pub kind: ElementKind,
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ElementKind {
    Core,
    L3,
}

impl Platform {
    pub fn new(config: &Config) -> Result<Platform> {
        let path = path!(config, "a stack description");

        info!(target: "platform", "Reading {:?}...", &path);
        let system = ok!(System::new(&path));

        let mut units = 0;
        let mut elements = HashMap::new();
        for element in system.stack.elements.iter().rev() {
            let die = match element {
                &StackElement::Die(ref die) => die,
                _ => continue,
            };
            for element in die.floorplan.elements.iter() {
                let kind = try!(ElementKind::from_str(&element.id));
                let id = ID::new("element");
                let heap = elements.entry(kind).or_insert_with(|| BinaryHeap::new());
                heap.push(time!(0.0, Element { id: id, kind: kind }));
                units += 1;
            }
        }

        let config = {
            let config = some!(config.branch("temperature"),
                               "a temperature configuration is required");
            try!(new_temperature_config(&config))
        };

        info!(target: "platform", "Constructing a thermal circuit...");
        let simulator = ok!(Simulator::new(&ok!(ThreeDICE::from(&system)), &config));

        Ok(Platform {
            units: units,
            elements: elements,
            simulator: simulator,
            power: Profile::new(units, config.time_step),
            temperature: Profile::new(units, config.time_step),
        })
    }

    pub fn push(&mut self, job: &Job) -> Result<(f64, f64)> {
        use std::f64::EPSILON;

        let pattern = &job.pattern;

        let units = pattern.units;
        if self.units < units {
            raise!("do not have enough resources for {}", job);
        }

        let mut available = 0f64;
        let mut hosts = vec![];
        for element in &pattern.elements {
            match self.elements.get_mut(&element.kind).and_then(|heap| heap.pop()) {
                Some(host) => {
                    if host.is_exclusive() {
                        available = available.max(host.time);
                    }
                    hosts.push(host);
                },
                _ => break,
            }
        }

        macro_rules! push_back(
            () => (for host in hosts.drain(..) {
                self.elements.get_mut(&host.kind).unwrap().push(host);
            });
        );

        if hosts.len() != units {
            push_back!();
            raise!("failed to allocate resources for {}", job);
        }

        let start = job.arrival.max(available) + EPSILON;
        let finish = start + pattern.duration();

        for i in 0..units {
            let element = &pattern.elements[i];
            let mut power = element.dynamic_power.clone();
            for power in &mut power {
                *power += element.leakage_power;
            }
            self.power.accumulate(hosts[i].number(), start, pattern.time_step, &power);
        }

        for host in &mut hosts {
            host.time = finish;
        }
        push_back!();

        Ok((start, finish))
    }
}

impl Element {
    #[inline(always)]
    pub fn number(&self) -> usize {
        self.id.number()
    }

    #[inline(always)]
    pub fn is_exclusive(&self) -> bool {
        self.kind == ElementKind::Core
    }
}

impl FromStr for ElementKind {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self> {
        let lower = id.to_lowercase();
        if lower.starts_with("core") {
            return Ok(ElementKind::Core);
        } else if lower.starts_with("l3") {
            return Ok(ElementKind::L3);
        }
        raise!("found an unknown id {:?}", id);
    }
}

fn new_temperature_config(config: &Config) -> Result<temperature::Config> {
    Ok(temperature::Config {
        ambience: *some!(config.get::<f64>("ambience"), "an ambient temperature is required"),
        time_step: *some!(config.get::<f64>("time_step"), "a time step is required"),
    })
}

#[cfg(test)]
mod tests {
    use config::Config;
    use super::{ElementKind, Platform};

    #[test]
    fn new() {
        let config = Config::new("tests/fixtures/streamer.toml").unwrap()
                            .branch("platform").unwrap();
        let platform = Platform::new(&config).unwrap();
        assert_eq!(platform.elements[&ElementKind::Core].len(), 4);
        assert_eq!(platform.elements[&ElementKind::L3].len(), 1);
    }
}
