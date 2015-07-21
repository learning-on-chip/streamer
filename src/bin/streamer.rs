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

macro_rules! raise(
    ($message:expr) => (return Err(Box::new(ErrorString($message.to_string()))));
);

macro_rules! ok(
    ($result:expr) => (match $result {
        Ok(result) => result,
        Err(error) => return Err(Box::new(error)),
    });
);

macro_rules! some(
    ($option:expr, $($arg:tt)*) => (match $option {
        Some(value) => value,
        _ => raise!($($arg)*),
    });
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

    let system = {
        let config = some!(arguments.get::<String>("config"), "a configuration file is required");
        try!(System::new(PathBuf::from(config), &random::default().seed([69, 42])))
    };

    for event in system.take(100) {
        println!("{}", event);
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
