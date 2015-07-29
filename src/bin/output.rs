use arguments::Arguments;
use streamer::{Increment, Result};

pub trait Output {
    fn next(&mut self, Increment) -> Result<()>;
}

pub struct Terminal;

impl Terminal {
    pub fn new(_: &Arguments) -> Result<Terminal> {
        Ok(Terminal)
    }
}

impl Output for Terminal {
    fn next(&mut self, (event, power, _): Increment) -> Result<()> {
        if power.steps > 0 {
            println!("{} - {} samples", event, power.steps);
        } else {
            println!("{}", event);
        }
        Ok(())
    }
}

pub fn new(arguments: &Arguments) -> Result<Box<Output>> {
    Ok(Box::new(try!(Terminal::new(arguments))))
}
