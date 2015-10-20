//! Multiprocessor platform.

use Result;
use schedule::Decision;
use system::Job;

mod element;
mod profile;
mod thermal;

pub use self::element::{Element, ElementCapacity, ElementKind};
pub use self::profile::{Profile, ProfileBuilder};
pub use self::thermal::Thermal;

/// A multiprocessor platform.
pub trait Platform {
    /// The data produced by the platform.
    type Data;

    /// Return the processing elements.
    fn elements(&self) -> &[Element];

    /// Advance time and return the data accumulated since the previous call.
    fn next(&mut self, f64) -> Result<Self::Data>;

    /// Account for a scheduling decision taken with respect to a job.
    fn push(&mut self, &Job, &Decision) -> Result<()>;
}
