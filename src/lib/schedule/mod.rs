//! Scheduling policy.

use Result;
use system::Job;

mod compact;
mod queue;

pub use self::compact::Compact;
pub use self::queue::{Interval, Queue};

/// A schedule.
pub trait Schedule {
    /// Take a decision with respect to a job.
    fn push(&mut self, &Job) -> Result<Decision>;

    /// Advance time.
    fn tick(&mut self, f64);
}

/// A scheduling decision.
#[derive(Clone, Debug)]
pub struct Decision {
    /// The start of the execution interval.
    pub start: f64,
    /// The end of the execution interval.
    pub finish: f64,
    /// The mapping from the processing elements of the job to the processing
    /// elements of the platform.
    pub mapping: Vec<(usize, usize)>,
}

impl Decision {
    /// Create a decision.
    #[inline]
    pub fn new(start: f64, finish: f64, mapping: Vec<(usize, usize)>) -> Decision {
        Decision { start: start, finish: finish, mapping: mapping }
    }
}
