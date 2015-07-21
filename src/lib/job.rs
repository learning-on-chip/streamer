use std::fmt;
use std::rc::Rc;

use id::ID;
use workload::Pattern;

#[derive(Clone, Debug)]
pub struct Job {
    pub id: ID,
    pub pattern: Rc<Pattern>,
}

impl Job {
    #[inline]
    pub fn new(pattern: Rc<Pattern>) -> Job {
        Job { id: ID::new("job"), pattern: pattern }
    }
}

impl fmt::Display for Job {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "job ({:5} {:15})", self.id, self.pattern.name)
    }
}
