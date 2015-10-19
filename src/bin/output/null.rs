use output::Output;
use {Data, Event, Result};

pub struct Null;

impl Output for Null {
    fn next(&mut self, _: &Event, _: &Data) -> Result<()> {
        Ok(())
    }
}
