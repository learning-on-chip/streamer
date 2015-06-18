use Result;
use config;
use std::path::Path;

mod sqlite;

pub struct Source {
    pub names: Vec<String>,
    pub dynamic: Vec<f64>,
    pub leakage: Vec<f64>,
}

pub fn new(config: &config::Source, root: &Path) -> Result<Source> {
    Ok(match config.kind {
        Some(ref kind) => match &**kind {
            "sqlite" => try!(sqlite::new(config, root)),
            _ => raise!("the source kind {:?} is unknown", kind),
        },
        _ => raise!("the source kind is required"),
    })
}
