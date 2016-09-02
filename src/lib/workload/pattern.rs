use std::rc::Rc;

use {Config, Result};
use workload::Component;

/// A workload pattern.
#[derive(Clone, Debug)]
pub struct Pattern(Rc<Content>);

deref! { Pattern::0 => Content }

/// The content of a workload pattern.
#[derive(Clone, Debug)]
pub struct Content {
    /// The name.
    pub name: String,
    /// The number of components.
    pub units: usize,
    /// The number of time steps.
    pub step_count: usize,
    /// The time step (sampling interval).
    pub time_step: f64,
    /// The components.
    pub components: Vec<Component>,
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
        let components = try!(Component::collect(&path));

        let units = components.len();
        if units == 0 {
            raise!("found a workload pattern without components");
        }
        let step_count = components[0].dynamic_power.len();
        if step_count == 0 {
            raise!("found a workload pattern without dynamic-power data");
        }

        Ok(Pattern(Rc::new(Content {
            name: name,
            units: units,
            step_count: step_count,
            time_step: time_step,
            components: components,
        })))
    }

    /// Return the duration.
    #[inline]
    pub fn duration(&self) -> f64 {
        self.step_count as f64 * self.time_step
    }
}
