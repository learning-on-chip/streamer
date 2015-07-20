use Result;
use config::Config;
use temperature::{self, Analysis};
use temperature::circuit::ThreeDICE;
use threed_ice::{StackElement, System};

pub struct Platform {
    pub analysis: Analysis,
    pub elements: Vec<Element>,
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

        info!(target: "platform", "Constructing a thermal circuit...");
        let analysis = match config.branch("temperature") {
            Some(ref temperature) => {
                let temperature = try!(new_temperature_config(temperature));
                ok!(Analysis::new(&ok!(ThreeDICE::from(&system)), &temperature))
            },
            _ => raise!("a temperature configuration is required"),
        };

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

        Ok(Platform { analysis: analysis, elements: elements })
    }
}

fn new_temperature_config(config: &Config) -> Result<temperature::Config> {
    Ok(temperature::Config {
        ambience: *expect!(config.get::<f64>("ambience"), "an ambient temperature"),
        time_step: *expect!(config.get::<f64>("time_step"), "a time step"),
    })
}

#[cfg(test)]
mod tests {
    use config::Config;
    use super::Element::Core;
    use super::Platform;

    #[test]
    fn new() {
        let config = Config::new("tests/fixtures/streamer.toml").unwrap()
                            .branch("platform").unwrap();
        let platform = Platform::new(&config).unwrap();
        assert_eq!(&platform.elements, &[Core, Core, Core, Core]);
    }
}
