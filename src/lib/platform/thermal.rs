use std::path::Path;
use std::str::FromStr;
use temperature::{self, Simulator};

use platform::{Element, ElementKind, Platform, Profile, ProfileBuilder};
use schedule::Decision;
use system::Job;
use {Config, Result};

/// A platform producing power and temperature data.
pub struct Thermal {
    elements: Vec<Element>,
    simulator: Simulator,
    builder: ProfileBuilder,
}

impl Thermal {
    /// Create a platform.
    pub fn new(config: &Config) -> Result<Thermal> {
        let (elements, simulator) = try!(construct_temperature(config.branch("temperature")
                                                                     .as_ref().unwrap_or(config)));

        let builder = try!(construct_power(&elements, config.branch("power").as_ref()
                                                            .unwrap_or(config)));

        Ok(Thermal { elements: elements, simulator: simulator, builder: builder })
    }
}

impl Platform for Thermal {
    type Data = (Profile, Profile);

    #[inline(always)]
    fn elements(&self) -> &[Element] {
        &self.elements
    }

    fn next(&mut self, time: f64) -> Option<Self::Data> {
        let power = self.builder.pull(time);
        let mut temperature = power.clone_zero();
        self.simulator.next(&power, &mut temperature);
        Some((power, temperature))
    }

    fn push(&mut self, job: &Job, decision: &Decision) -> Result<()> {
        let pattern = &job.pattern;
        let (from, onto) = (&pattern.elements, &self.elements);
        for &(i, j) in &decision.mapping {
            let (from, onto) = (&from[i], &onto[j]);
            self.builder.push(onto.id, decision.start, pattern.time_step, &from.dynamic_power);
        }
        Ok(())
    }
}

fn construct_power(elements: &[Element], config: &Config) -> Result<ProfileBuilder> {
    use workload;

    let units = elements.len();
    let time_step = *some!(config.get::<f64>("time_step"), "a time step is required");

    let path = path!(config, "a leakage pattern is required");
    info!(target: "Platform", "Modeling leakage power based on {:?}...", &path);
    let candidates = try!(workload::Element::collect(path));

    let mut leakage_power = vec![0.0; units];
    for i in 0..units {
        if let Some(element) = candidates.iter().find(|element| element.kind == elements[i].kind) {
            leakage_power[i] = element.leakage_power;
        } else {
            raise!("cannot find leakage data for a processing element");
        }
    }

    Ok(ProfileBuilder::new(units, time_step, leakage_power))
}

fn construct_temperature(config: &Config) -> Result<(Vec<Element>, Simulator)> {
    let path = path!(config, "a thermal specification is required");
    info!(target: "Platform", "Modeling temperature based on {:?}...", &path);
    let (elements, circuit) = match path.extension() {
        Some(extension) if extension == "stk" => try!(construct_threed_ice(&path)),
        _ => raise!("the format of {:?} is unknown", &path),
    };
    info!(target: "Platform", "Found {} processing elements and {} thermal nodes.",
          elements.len(), circuit.capacitance.len());

    info!(target: "Platform", "Initializing the temperature simulator...");
    let config = temperature::Config {
        ambience: *some!(config.get::<f64>("ambience"), "an ambient temperature is required"),
        time_step: *some!(config.get::<f64>("time_step"), "a time step is required"),
    };
    let simulator = ok!(Simulator::new(circuit, config));

    Ok((elements, simulator))
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
            let kind = try!(ElementKind::from_str(&element.id));
            elements.push(Element { id: id, kind: kind });
        }
    }
    Ok((elements, ok!(ThreeDICE::from(&system))))
}

#[cfg(test)]
mod tests {
    use configuration::format::TOML;
    use platform::{Element, ElementKind, Thermal};

    #[test]
    fn new() {
        let config = TOML::open("tests/fixtures/streamer.toml").unwrap()
                                                               .branch("platform")
                                                               .unwrap();
        let platform = Thermal::new(&config).unwrap();
        assert_eq!(platform.elements, &[
            Element { id: 0, kind: ElementKind::Core },
            Element { id: 1, kind: ElementKind::Core },
            Element { id: 2, kind: ElementKind::Core },
            Element { id: 3, kind: ElementKind::Core },
            Element { id: 4, kind: ElementKind::L3 },
        ]);
    }
}
