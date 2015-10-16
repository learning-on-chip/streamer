use {Job, Result};

pub trait Schedule {
    fn push(&mut self, &Job) -> Result<Decision>;
    fn pass(&mut self, f64);
}

pub struct Decision {
    pub start: f64,
    pub finish: f64,
    pub mapping: Vec<(usize, usize)>,
}

mod compact;
mod queue;

pub use self::compact::Compact;

impl Decision {
    #[inline]
    pub fn new(start: f64, finish: f64, mapping: Vec<(usize, usize)>) -> Decision {
        Decision { start: start, finish: finish, mapping: mapping }
    }
}
