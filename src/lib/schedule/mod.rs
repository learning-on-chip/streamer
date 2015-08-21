use {Job, Result};

pub trait Schedule {
    fn push(&mut self, &Job) -> Result<(f64, f64, Vec<(usize, usize)>)>;
}

mod compact;
mod queue;

pub use self::compact::Compact;
