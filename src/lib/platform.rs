use std::str::FromStr;
use temperature::circuit::ThreeDICE;
use temperature::{self, Simulator};
use threed_ice::{StackElement, System};

use profile::Profile;
use schedule::{self, Schedule};
use {Config, Error, Job, Result};

pub struct Platform {
    pub elements: Vec<Element>,
    pub schedule: Box<Schedule>,
    pub simulator: Simulator,
    pub power: Profile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Element {
    pub id: usize,
    pub kind: ElementKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ElementKind {
    Core,
    L3,
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
                let kind = try!(ElementKind::from_str(&element.id));
                elements.push(Element { id: id, kind: kind });
            }
        }
        info!(target: "Platform", "Found {} processing elements.", elements.len());

        let schedule = Box::new(try!(schedule::Compact::new(&elements)));

        let config = {
            let config = some!(config.branch("temperature"),
                               "a temperature configuration is required");
            try!(new_temperature_config(&config))
        };

        info!(target: "Platform", "Constructing a thermal circuit...");
        let circuit = ok!(ThreeDICE::from(&system));
        info!(target: "Platform", "Obtained {} thermal nodes.", circuit.capacitance.len());

        info!(target: "Platform", "Initializing the temperature simulator...");
        let simulator = ok!(Simulator::new(&circuit, &config));

        let power = Profile::new(elements.len(), config.time_step);

        Ok(Platform {
            elements: elements,
            schedule: schedule,
            simulator: simulator,
            power: power,
        })
    }

    pub fn push(&mut self, job: &Job) -> Result<(f64, f64)> {
        let (start, finish, mapping) = try!(self.schedule.push(job));
        let (from, onto) = (&job.pattern.elements, &self.elements);
        for (i, j) in mapping {
            let (from, onto) = (&from[i], &onto[j]);
            self.power.accumulate(onto.id, start, job.pattern.time_step, &from.dynamic_power,
                                  from.leakage_power);
        }
        Ok((start, finish))
    }

    pub fn next(&mut self, time: f64) -> Option<(Profile, Profile)> {
        let power = self.power.discharge(time);
        let mut temperature = power.clone_zero();
        self.simulator.step(&power, &mut temperature);
        Some((power, temperature))
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    #[inline]
    pub fn time_step(&self) -> f64 {
        self.power.time_step
    }
}

impl Element {
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
    use super::{Element, ElementKind, Platform};

    #[test]
    fn new() {
        let config = Config::new("tests/fixtures/streamer.toml").unwrap()
                            .branch("platform").unwrap();
        let platform = Platform::new(&config).unwrap();
        assert_eq!(platform.elements, &[
            Element { id: 0, kind: ElementKind::Core },
            Element { id: 1, kind: ElementKind::Core },
            Element { id: 2, kind: ElementKind::Core },
            Element { id: 3, kind: ElementKind::Core },
            Element { id: 4, kind: ElementKind::L3 },
        ]);
    }
}
