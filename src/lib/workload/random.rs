use probability::distribution::{Categorical, Sample};

use workload::{Pattern, Workload};
use {Config, Result, Source};

pub struct Random {
    patterns: Vec<Pattern>,
    source: Source,
    distribution: Categorical,
}

impl Random {
    pub fn new(config: &Config, source: &Source) -> Result<Random> {
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
            source: source.clone(),
            distribution: Categorical::new(&vec![1.0 / count as f64; count]),
        })
    }
}

impl Workload for Random {
    fn next(&mut self) -> Option<Pattern> {
        Some(self.patterns[self.distribution.sample(&mut self.source)].clone())
    }
}
