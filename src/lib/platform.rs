use Result;
use config::Config;
use temperature::{self, Simulator};
use temperature::circuit::ThreeDICE;
use threed_ice::{StackElement, System};

pub struct Platform {
    pub elements: Vec<Element>,
    pub temperature: Simulator,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Element {
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
            if let &StackElement::Die(ref die) = element {
                for element in die.floorplan.elements.iter() {
                    let id = element.id.to_lowercase();
                    if id.starts_with("core") {
                        elements.push(Element::Core);
                    } else if id.starts_with("l3") {
                        elements.push(Element::L3);
                    } else {
                        raise!("found an unknown id {:?}", &element.id);
                    }
                }
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

fn new_temperature_config(config: &Config) -> Result<temperature::Config> {
    Ok(temperature::Config {
        ambience: *some!(config.get::<f64>("ambience"), "an ambient temperature is required"),
        time_step: *some!(config.get::<f64>("time_step"), "a time step is required"),
    })
}

#[cfg(test)]
mod tests {
    use config::Config;
    use super::Element::{Core, L3};
    use super::Platform;

    #[test]
    fn new() {
        let config = Config::new("tests/fixtures/streamer.toml").unwrap()
                            .branch("platform").unwrap();
        let platform = Platform::new(&config).unwrap();
        assert_eq!(&platform.elements, &[Core, Core, Core, Core, L3]);
    }
}
