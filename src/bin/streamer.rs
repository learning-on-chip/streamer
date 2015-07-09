#![cfg_attr(test, allow(dead_code))]

extern crate arguments;
extern crate log;
extern crate random;
extern crate streamer;

use std::error::Error;
use std::path::PathBuf;
use streamer::{ErrorString, Result, System};

mod logger;

const USAGE: &'static str = "
Usage: streamer [options]

Options:
    --config <path>          Configuration file (required).

    --help                   Display this message.
";

macro_rules! ok(
    ($result:expr) => (match $result {
        Ok(result) => result,
        Err(error) => return Err(Box::new(error)),
    });
);

macro_rules! raise(
    ($message:expr) => (return Err(Box::new(ErrorString($message.to_string()))));
);

fn main() {
    logger::setup();
    start().unwrap_or_else(|error| fail(&*error));
}

fn start() -> Result<()> {
    let arguments = ok!(arguments::parse(std::env::args()));

    if arguments.get::<bool>("help").unwrap_or(false) {
        help();
    }

    let system = match arguments.get::<String>("config").map(|config| PathBuf::from(config)) {
        Some(ref config) => try!(System::new(config, &random::default().seed([69, 42]))),
        _ => raise!("a configuration file is required"),
    };

    for state in system.take(100) {
        println!("{}", state);
    }

    Ok(())
}

fn help() -> ! {
    println!("{}", USAGE.trim());
    std::process::exit(0);
}

fn fail(error: &Error) -> ! {
    use std::io::{stderr, Write};
    stderr().write_all(format!("Error: {}.\n", error).as_bytes()).unwrap();
    std::process::exit(1);
}
