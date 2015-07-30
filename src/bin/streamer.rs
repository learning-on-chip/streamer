extern crate arguments;
extern crate log;
extern crate random;
extern crate sqlite;
extern crate streamer;
extern crate term;

use log::LogLevel;
use std::error::Error;
use std::path::PathBuf;
use streamer::{Result, System};

const USAGE: &'static str = "
Usage: streamer [options]

Options:
    --config <path>          Configuration file (required).
    --length <time>          Time span to simulate in seconds [default: 10].
    --output <path>          Output file for power and temperature profiles.

    --verbose                Display progress information.
    --help                   Display this message.
";

macro_rules! raise(
    ($message:expr) => (return Err(Box::new(::streamer::ErrorString($message.to_string()))));
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

mod logger;
mod output;

use output::Output;

fn main() {
    start().unwrap_or_else(|error| fail(&*error));
}

fn start() -> Result<()> {
    let arguments = ok!(arguments::parse(std::env::args()));

    if arguments.get::<bool>("help").unwrap_or(false) {
        help();
    }

    if arguments.get::<bool>("verbose").unwrap_or(false) {
        logger::setup(LogLevel::Info);
    } else {
        logger::setup(LogLevel::Warn);
    }

    let system = {
        let config = some!(arguments.get::<String>("config"), "a configuration file is required");
        try!(System::new(PathBuf::from(config), &random::default().seed([69, 42])))
    };

    let length = arguments.get::<f64>("length").unwrap_or(10.0);
    if length <= 0.0 {
        raise!("the time span should be positive");
    }

    let mut output = try!(output::new(&system, arguments.get::<String>("output")));

    for (event, power, temperature) in system {
        if event.time > length {
            break;
        }
        try!(output.next((event, power, temperature)));
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
