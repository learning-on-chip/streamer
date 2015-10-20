//! Workload modeling.

use Result;

mod element;
mod pattern;
mod random;

pub use self::element::Element;
pub use self::pattern::{Content, Pattern};
pub use self::random::Random;

/// A workload model.
pub trait Workload {
    /// Assign a workload pattern to a job arrival.
    fn next(&mut self, f64) -> Result<Pattern>;
}
