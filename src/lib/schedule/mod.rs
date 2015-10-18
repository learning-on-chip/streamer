use {Job, Result};

mod compact;
mod queue;

pub use self::compact::Compact;

pub trait Schedule {
    fn push(&mut self, &Job) -> Result<Decision>;
    fn tick(&mut self, f64);
}

#[derive(Clone, Debug)]
pub struct Decision {
    pub start: f64,
    pub finish: f64,
    pub mapping: Vec<(usize, usize)>,
}

impl Decision {
    #[inline]
    pub fn new(start: f64, finish: f64, mapping: Vec<(usize, usize)>) -> Decision {
        Decision { start: start, finish: finish, mapping: mapping }
    }
}
