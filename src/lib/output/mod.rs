//! Data output.

use Result;
use system::Event;

mod thermal;

pub use self::thermal::Thermal;

/// An output.
pub trait Output {
    type Data;

    fn next(&mut self, &Event, &Self::Data) -> Result<()>;
}
