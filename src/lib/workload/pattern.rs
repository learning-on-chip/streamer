use std::rc::Rc;

use workload::Element;
use {Config, Result};

/// A workload pattern.
#[derive(Clone, Debug)]
pub struct Pattern(Rc<Content>);

deref! { Pattern::0 => Content }

/// The content of a workload pattern.
#[derive(Clone, Debug)]
pub struct Content {
    /// The name.
    pub name: String,
    /// The number of processing elements.
    pub units: usize,
    /// The number of time steps.
    pub steps: usize,
    /// The time step (sampling interval).
    pub time_step: f64,
    /// The processing elements.
    pub elements: Vec<Element>,
}

impl Pattern {
    /// Create a pattern.
    pub fn new(config: &Config) -> Result<Pattern> {
        let path = path!(config, "a workload-pattern database is required");

        let name = match config.get::<String>("name") {
            Some(name) => name.to_string(),
            _ => path.file_stem().unwrap().to_str().unwrap().to_string(),
        };
        let time_step = *some!(config.get::<f64>("time_step"), "a time step is required");

        info!(target: "Workload", "Reading a pattern from {:?}...", &path);
        let elements = try!(Element::collect(&path));

        let units = elements.len();
        if units == 0 {
            raise!("found a workload pattern without processing elements");
        }
        let steps = elements[0].dynamic_power.len();
        if steps == 0 {
            raise!("found a workload pattern without dynamic-power data");
        }

        Ok(Pattern(Rc::new(Content {
            name: name,
            units: units,
            steps: steps,
            time_step: time_step,
            elements: elements,
        })))
    }

    /// Return the time duration.
    #[inline]
    pub fn duration(&self) -> f64 {
        self.steps as f64 * self.time_step
    }
}
