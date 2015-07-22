use std::collections::{BinaryHeap, HashMap};
use std::str::FromStr;
use temperature::circuit::ThreeDICE;
use temperature::{self, Simulator};
use threed_ice::{StackElement, System};

use config::Config;
use {Error, ID, Job, Result};

pub struct Platform {
    pub elements: HashMap<ElementKind, BinaryHeap<Element>>,
    pub temperature: Simulator,
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

        let mut elements = HashMap::new();
        for element in system.stack.elements.iter().rev() {
            let die = match element {
                &StackElement::Die(ref die) => die,
                _ => continue,
            };
            for element in die.floorplan.elements.iter() {
                let kind = try!(ElementKind::from_str(&element.id));
                let id = ID::new(kind.as_str());
                let heap = elements.entry(kind).or_insert_with(|| BinaryHeap::new());
                heap.push(time!(0.0, Element { id: id, kind: kind }));
            }
        }

        info!(target: "platform", "Constructing a thermal circuit...");
        let temperature = {
            let config = some!(config.branch("temperature"),
                               "a temperature configuration is required");
            ok!(Simulator::new(&ok!(ThreeDICE::from(&system)),
                               &try!(new_temperature_config(&config))))
        };

        Ok(Platform { elements: elements, temperature: temperature })
    }

    pub fn next(&mut self, job: Job) -> Option<f64> {
        let _ = job.pattern.span();

        for element in &job.pattern.elements {
            for _ in &self.elements[&element.kind] {
            }
        }

        None
    }
}

impl ElementKind {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ElementKind::Core => "core",
            ElementKind::L3 => "l3",
        }
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
