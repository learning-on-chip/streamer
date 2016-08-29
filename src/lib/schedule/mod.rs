//! Job scheduling.

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
    fn next(&mut self, &Job) -> Result<Decision>;

    /// Advance time and consume the data accumulated since the previous call.
    fn push(&mut self, f64, Self::Data) -> Result<()>;
}

/// A scheduling decision.
#[derive(Clone, Debug)]
pub enum Decision {
    Accept {
        /// The start of the execution interval.
        start: f64,
        /// The end of the execution interval.
        finish: f64,
        /// The mapping of the job to the platform.
        mapping: Mapping,
    },
    Reject,
}

/// A mapping of a job’s processing elements to the platform‘s procesing
/// elements.
pub type Mapping = Vec<(usize, usize)>;

/// A placeholder signifying that no data are needed.
#[derive(Clone, Copy)]
pub struct NoData;

impl Decision {
    /// Create an accept decision.
    #[inline]
    pub fn accept(start: f64, finish: f64, mapping: Mapping) -> Decision {
        Decision::Accept { start: start, finish: finish, mapping: mapping }
    }

    /// Create a reject decision.
    #[inline]
    pub fn reject() -> Decision {
        Decision::Reject
    }
}

impl<'l, T> From<&'l T> for NoData {
    #[inline]
    fn from(_: &'l T) -> NoData {
        NoData
    }
}
