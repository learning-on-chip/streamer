use std::path::Path;

use {Increment, Result, System};

mod database;
mod null;

use self::database::Database;
use self::null::Null;

pub trait Output {
    fn next(&mut self, Increment) -> Result<()>;
}

pub fn new<T: AsRef<Path>>(system: &System, output: Option<T>) -> Result<Box<Output>> {
    Ok(match output {
        Some(output) => Box::new(try!(Database::new(system, output))),
        _ => Box::new(Null),
    })
}
