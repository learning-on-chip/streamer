use Result;
use schedule::Decision;
use system::Job;

mod element;
mod profile;
mod thermal;

pub use self::element::{Capacity, Element, ElementKind};
pub use self::profile::Profile;
pub use self::thermal::Thermal;

pub trait Platform {
    type Output;

    fn elements(&self) -> &[Element];
    fn push(&mut self, &Job, &Decision) -> Result<()>;
    fn next(&mut self, f64) -> Option<Self::Output>;
}
