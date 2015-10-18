mod pattern;
mod random;

pub use self::pattern::{Content, Element, Pattern};
pub use self::random::Random;

pub trait Workload {
    fn next(&mut self, f64) -> Option<Pattern>;
}
