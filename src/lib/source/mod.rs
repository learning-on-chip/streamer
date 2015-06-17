use Result;
use config;
use std::path::Path;

mod sqlite;

pub use self::sqlite::SQLite;

pub trait Source {
}

pub fn new(config: &config::Source, root: &Path) -> Result<Box<Source>> {
    Ok(match config.kind {
        Some(ref kind) => match &**kind {
            "sqlite" => Box::new(try!(SQLite::new(config, root))),
            _ => raise!("the source kind {:?} is unknown", kind),
        },
        _ => raise!("the source kind is required"),
    })
}
