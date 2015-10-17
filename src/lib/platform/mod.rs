use std::path::Path;
use std::str::FromStr;
use temperature::{self, Simulator};

use schedule::Decision;
use {Config, Job, Result};

mod element;
mod profile;

pub use self::element::{Capacity, Element, Kind};
pub use self::profile::Profile;

pub struct Platform {
    elements: Vec<Element>,
    simulator: Simulator,
    power: Profile,
}

impl Platform {
    pub fn new(config: &Config) -> Result<Platform> {
        let (elements, circuit) = try!(construct_cirucit(config));
        info!(target: "Platform", "Found {} processing elements and {} thermal nodes.",
              elements.len(), circuit.capacitance.len());

        info!(target: "Platform", "Initializing the temperature simulator...");
        let config = try!(extract_temperature_config(config));
        let simulator = ok!(Simulator::new(&circuit, &config));

        let power = Profile::new(elements.len(), config.time_step);

        Ok(Platform { elements: elements, simulator: simulator, power: power })
    }

    pub fn push(&mut self, job: &Job, decision: &Decision) -> Result<()> {
        let pattern = &job.pattern;
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

    getters! {
        ref elements: [Element],
    }
}

fn construct_cirucit(config: &Config) -> Result<(Vec<Element>, temperature::Circuit)> {
    let path = path!(config, "a thermal specification is required");
    info!(target: "Platform", "Constructing a thermal circuit based on {:?}...", &path);
    match path.extension() {
        Some(extension) if extension == "stk" => construct_threed_ice(&path),
        _ => raise!("the format of {:?} is unknown", &path),
    }
}

fn construct_threed_ice(path: &Path) -> Result<(Vec<Element>, temperature::Circuit)> {
    use temperature::circuit::ThreeDICE;
    use threed_ice::{StackElement, System};

    let system = ok!(System::new(path));
    let mut elements = vec![];
    for element in system.stack.elements.iter().rev() {
        let die = match element {
            &StackElement::Die(ref die) => die,
            _ => continue,
        };
        for element in die.floorplan.elements.iter() {
            let id = elements.len();
            let kind = try!(Kind::from_str(&element.id));
            elements.push(Element { id: id, kind: kind });
        }
    }
    Ok((elements, ok!(ThreeDICE::from(&system))))
}

fn extract_temperature_config(config: &Config) -> Result<temperature::Config> {
    Ok(temperature::Config {
        ambience: *some!(config.get::<f64>("ambience"), "an ambient temperature is required"),
        time_step: *some!(config.get::<f64>("time_step"), "a time step is required"),
    })
}

#[cfg(test)]
mod tests {
    use configuration::format::TOML;
    use platform::{Kind, Element, Platform};

    #[test]
    fn new() {
        let config = TOML::open("tests/fixtures/streamer.toml").unwrap()
                                                               .branch("platform")
                                                               .unwrap();
        let platform = Platform::new(&config).unwrap();
        assert_eq!(platform.elements, &[
            Element { id: 0, kind: Kind::Core },
            Element { id: 1, kind: Kind::Core },
            Element { id: 2, kind: Kind::Core },
            Element { id: 3, kind: Kind::Core },
            Element { id: 4, kind: Kind::L3 },
        ]);
    }
}
