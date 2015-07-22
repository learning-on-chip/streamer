use std::fmt;

use ID;
use workload::Pattern;

rc!{
    #[derive(Clone, Debug)]
    pub struct Job(Content) {
        pub id: ID,
        pub pattern: Pattern,
    }
}

impl Job {
    #[inline]
    pub fn new(pattern: Pattern) -> Job {
        rc!(Job(Content { id: ID::new("job"), pattern: pattern }))
    }
}

impl fmt::Display for Job {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "job ({:5} {:15})", self.id, self.pattern.name)
    }
}
