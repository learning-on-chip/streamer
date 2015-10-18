//! Workload modeling.

mod pattern;
mod random;

pub use self::pattern::{Content, Element, Pattern};
pub use self::random::Random;

/// A workload model.
pub trait Workload {
    /// Assign a workload pattern to a job arrival.
    fn next(&mut self, f64) -> Option<Pattern>;
}
