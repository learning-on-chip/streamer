use Result;
use config::Config;
use threed_ice::{StackElement, System};

#[derive(Clone, Debug)]
pub struct Platform {
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

        Ok(Platform { elements: elements })
    }
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
