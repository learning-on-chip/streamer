use std::fs::File;
use std::path::Path;

use Result;

#[derive(RustcDecodable)]
pub struct Config {
    pub period: f64,
    pub sources: Vec<Source>,
}

#[derive(RustcDecodable)]
pub struct Source {
    pub name: String,
    pub kind: String,
    pub path: String,
}

pub fn open(path: &Path) -> Result<Config> {
    use rustc_serialize::Decodable;
    use std::io::Read;
    use toml::{Decoder, Parser, Value};

    let mut contents = String::new();
    ok!(ok!(File::open(path)).read_to_string(&mut contents));

    let config: Config = match Parser::new(&contents).parse() {
        Some(root) => ok!(Decodable::decode(&mut Decoder::new(Value::Table(root)))),
        _ => raise!("failed to parse the configuration file"),
    };

    Ok(config)
}
