use {Job, Result};

pub trait Schedule {
    fn push(&mut self, &Job) -> Result<(f64, f64, Vec<(usize, usize)>)>;
    fn trim(&mut self, f64);
}

mod compact;
mod queue;

pub use self::compact::Compact;
