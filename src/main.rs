#![cfg_attr(test, allow(dead_code))]

extern crate arguments;
extern crate rustc_serialize;
extern crate sqlite;
extern crate toml;

use std::path::PathBuf;

const USAGE: &'static str = "
Usage: streamer [options]

Options:
    --config <path>          Configuration file (required).

    --help                   Display this message.
";

macro_rules! raise(
    ($error:expr) => (return Err(Box::new($error)));
    ($($arg:tt)*) => (raise!(format!($($arg)*)));
);

macro_rules! ok(
    ($result:expr) => (match $result {
        Ok(result) => result,
        Err(error) => raise!(error),
    });
);

mod config;
mod source;

pub type Error = Box<std::fmt::Display>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() {
    start().unwrap_or_else(|error| fail(error));
}

fn start() -> Result<()> {
    let arguments = ok!(arguments::parse(std::env::args()));

    if arguments.get::<bool>("help").unwrap_or(false) {
        help();
    }

    let (root, config) = match arguments.get::<String>("config")
                                        .map(|config| PathBuf::from(config)) {
        Some(ref config) => {
            (config.parent().map(|path| PathBuf::from(path)), try!(config::open(config)))
        },
        _ => raise!("a configuration file is required"),
    };

    let mut sources = Vec::new();
    for &config::Source { ref kind, ref path, .. } in config.sources.iter() {
        let mut path = PathBuf::from(path);
        if path.is_relative() {
            if let Some(ref root) = root {
                path = root.join(path);
            }
        }
        if std::fs::metadata(&path).is_err() {
            raise!("the source file {:?} does not exist", &path);
        }
        match &**kind {
            "sqlite3" => sources.push(Box::new(ok!(sqlite::open(&path))) as Box<source::Source>),
            _ => raise!("the source kind {:?} is unknown", kind),
        }
    }
    if sources.is_empty() {
        raise!("at least one source is required");
    }

    Ok(())
}

fn help() -> ! {
    println!("{}", USAGE.trim());
    std::process::exit(0);
}

fn fail(error: Error) -> ! {
    use std::io::{stderr, Write};
    stderr().write_all(format!("Error: {}.\n", &*error).as_bytes()).unwrap();
    std::process::exit(1);
}
