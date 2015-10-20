//! Scheduling policy.

use Result;
use system::Job;

mod impartial;
mod queue;

pub use self::impartial::Impartial;
pub use self::queue::{Interval, Queue};

/// A scheduling policy.
pub trait Schedule {
    /// The data consumed by the policy.
    type Data;

    /// Take a decision with respect to a job.
    fn push(&mut self, &Job) -> Result<Decision>;

    /// Advance time and consume the accumulated data.
    fn step(&mut self, f64, &Self::Data) -> Result<()>;
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
