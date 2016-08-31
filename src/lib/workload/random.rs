use probability::distribution::{Categorical, Sample};

use {Config, Result, Source};
use workload::{Pattern, Workload};

/// A workload model that chooses workload patterns randomly.
pub struct Random {
    patterns: Vec<Pattern>,
    distribution: Categorical,
    source: Source,
}

impl Random {
    /// Create a model.
    pub fn new(config: &Config, source: Source) -> Result<Random> {
        let mut patterns = vec![];
        if let Some(ref configs) = config.forest("patterns") {
            for config in configs {
                patterns.push(try!(Pattern::new(config)));
            }
        }
        let count = patterns.len();
        if count == 0 {
            raise!("at least one workload pattern is required");
        }
        Ok(Random {
            patterns: patterns,
            distribution: Categorical::new(&vec![1.0 / count as f64; count]),
            source: source,
        })
    }
}

impl Workload for Random {
    fn next(&mut self, _: f64) -> Result<Pattern> {
        Ok(self.patterns[self.distribution.sample(&mut self.source)].clone())
    }
}
