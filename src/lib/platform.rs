use temperature::circuit::ThreeDICE;
use temperature::{self, Simulator};
use threed_ice::{StackElement, System};
use std::str::FromStr;

use config::Config;
use {Error, ID, Job, Result};

pub struct Platform {
    pub elements: Vec<Element>,
    pub temperature: Simulator,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Element {
    pub id: ID,
    pub kind: ElementKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementKind {
    Core,
    L3,
}

impl Platform {
    pub fn new(config: &Config) -> Result<Platform> {
        let path = path!(config, "a stack description");

        info!(target: "platform", "Reading {:?}...", &path);
        let system = ok!(System::new(&path));

        let mut elements = vec![];
        for element in system.stack.elements.iter().rev() {
            let die = match element {
                &StackElement::Die(ref die) => die,
                _ => continue,
            };
            for element in die.floorplan.elements.iter() {
                elements.push(Element {
                    id: ID::new("element"),
                    kind: try!(element.id.parse()),
                });
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
    use super::ElementKind::{Core, L3};
    use super::Platform;

    #[test]
    fn new() {
        let config = Config::new("tests/fixtures/streamer.toml").unwrap()
                            .branch("platform").unwrap();
        let platform = Platform::new(&config).unwrap();
        assert_eq!(&platform.elements.iter().map(|element| element.kind).collect::<Vec<_>>(),
                   &[Core, Core, Core, Core, L3]);
    }
}
