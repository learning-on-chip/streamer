#![cfg_attr(test, allow(dead_code))]

extern crate arguments;
#[macro_use] extern crate streamer;

use std::path::PathBuf;
use streamer::{Error, Result, Streamer};

const USAGE: &'static str = "
Usage: streamer [options]

Options:
    --config <path>          Configuration file (required).

    --help                   Display this message.
";

fn main() {
    start().unwrap_or_else(|error| fail(error));
}

fn start() -> Result<()> {
    let arguments = ok!(arguments::parse(std::env::args()));

    if arguments.get::<bool>("help").unwrap_or(false) {
        help();
    }
    let _streamer = match arguments.get::<String>("config").map(|config| PathBuf::from(config)) {
        Some(ref config) => try!(Streamer::new(config)),
        _ => raise!("a configuration file is required"),
    };

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
