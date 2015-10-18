//! Multiprocessor platform.

use Result;
use schedule::Decision;
use system::Job;

mod element;
mod profile;
mod thermal;

pub use self::element::{Element, ElementCapacity, ElementKind};
pub use self::profile::Profile;
pub use self::thermal::Thermal;

/// A multiprocessor platform.
pub trait Platform {
    /// The data produced by the platform.
    type Data;

    /// Return the processing elements.
    fn elements(&self) -> &[Element];

    /// Advance time and return the accumulated data.
    fn next(&mut self, f64) -> Option<Self::Data>;

    /// Account for a scheduling decision taken with respect to a job.
    fn push(&mut self, &Job, &Decision) -> Result<()>;
}
