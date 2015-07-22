use std::fmt;

use ID;
use workload::Pattern;

rc! {
    #[derive(Clone, Debug)]
    pub struct Job(Content) {
        pub id: ID,
        pub arrival: f64,
        pub pattern: Pattern,
    }
}

impl Job {
    #[inline]
    pub fn new(arrival: f64, pattern: Pattern) -> Job {
        rc!(Job(Content { id: ID::new("job"), arrival: arrival, pattern: pattern }))
    }
}

impl fmt::Display for Job {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Job {} ({})", self.id, self.pattern.name)
    }
}
