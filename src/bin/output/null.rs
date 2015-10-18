use output::Output;
use {Increment, Result};

pub struct Null;

impl Output for Null {
    fn next(&mut self, _: Increment) -> Result<()> {
        Ok(())
    }
}
