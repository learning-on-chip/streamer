use std::str::FromStr;
use temperature::circuit::ThreeDICE;
use temperature::{self, Simulator};
use threed_ice::{StackElement, System};

use profile::Profile;
use schedule::Decision;
use {Config, Error, Job, Result};

pub struct Platform {
    elements: Vec<Element>,
    simulator: Simulator,
    power: Profile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Element {
    pub id: usize,
    pub class: Class,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Class {
    Core,
    L3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capacity {
    Single,
    Infinite,
}

impl Platform {
    pub fn new(config: &Config) -> Result<Platform> {
        let path = path!(config, "a stack description");

        info!(target: "Platform", "Reading {:?}...", &path);
        let system = ok!(System::new(&path));

        let mut elements = vec![];
        for element in system.stack.elements.iter().rev() {
            let die = match element {
                &StackElement::Die(ref die) => die,
                _ => continue,
            };
            for element in die.floorplan.elements.iter() {
                let id = elements.len();
                let class = try!(Class::from_str(&element.id));
                elements.push(Element { id: id, class: class });
            }
        }
        info!(target: "Platform", "Found {} processing elements.", elements.len());

        info!(target: "Platform", "Constructing a thermal circuit...");
        let circuit = ok!(ThreeDICE::from(&system));
        info!(target: "Platform", "Obtained {} thermal nodes.", circuit.capacitance.len());

        info!(target: "Platform", "Initializing the temperature simulator...");
        let config = try!(new_temperature_config(&config));
        let simulator = ok!(Simulator::new(&circuit, &config));

        let power = Profile::new(elements.len(), config.time_step);

        Ok(Platform { elements: elements, simulator: simulator, power: power })
    }

    pub fn push(&mut self, job: &Job, decision: &Decision) -> Result<()> {
        let pattern = job.pattern();
        let (from, onto) = (&pattern.elements, &self.elements);
        for &(i, j) in &decision.mapping {
            let (from, onto) = (&from[i], &onto[j]);
            self.power.push(onto.id, decision.start, pattern.time_step, &from.dynamic_power,
                            from.leakage_power);
        }
        Ok(())
    }

    pub fn next(&mut self, time: f64) -> Option<(Profile, Profile)> {
        let power = self.power.pull(time);
        let mut temperature = power.clone_zero();
        self.simulator.next(&power, &mut temperature);
        Some((power, temperature))
    }

    getter! { ref elements: [Element] }
}

impl Element {
    #[inline(always)]
    pub fn capacity(&self) -> Capacity {
        if self.class == Class::Core {
            Capacity::Single
        } else {
            Capacity::Infinite
        }
    }
}

impl FromStr for Class {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self> {
        let lower = id.to_lowercase();
        if lower.starts_with("core") {
            return Ok(Class::Core);
        } else if lower.starts_with("l3") {
            return Ok(Class::L3);
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
    use configure;
    use super::{Class, Element, Platform};

    #[test]
    fn new() {
        let config = configure("tests/fixtures/streamer.toml").unwrap()
                                                              .branch("platform")
                                                              .unwrap();
        let platform = Platform::new(&config).unwrap();
        assert_eq!(platform.elements, &[
            Element { id: 0, class: Class::Core },
            Element { id: 1, class: Class::Core },
            Element { id: 2, class: Class::Core },
            Element { id: 3, class: Class::Core },
            Element { id: 4, class: Class::L3 },
        ]);
    }
}
