use Result;
use config::Config;
use threed_ice::{StackElement, System};

pub struct Platform {
    pub elements: Vec<Element>,
}

pub struct Element {
    pub name: String,
}

impl Platform {
    pub fn new(config: &Config) -> Result<Platform> {
        let path = path!(config, "a stack description");

        info!(target: "platform", "Reading {:?}...", &path);
        let system = ok!(System::new(&path));

        let mut elements = vec![];
        for element in system.stack_description().elements() {
            if let StackElement::Die(ref die) = element {
                for element in die.floorplan.elements.iter() {
                    elements.push(Element {
                        name: element.name.clone(),
                    });
                }
            }
        }

        Ok(Platform { elements: elements })
    }
}

#[cfg(test)]
mod tests {
    use config::Config;
    use super::Platform;

    #[test]
    fn new() {
        let config = Config::new("tests/fixtures/streamer.toml").unwrap()
                            .branch("platform").unwrap();
        let platform = Platform::new(&config).unwrap();
        assert_eq!(platform.elements.iter().map(|element| &element.name).collect::<Vec<_>>(),
                   &["Core0", "Core1", "Core2", "Core3"]);
    }
}
