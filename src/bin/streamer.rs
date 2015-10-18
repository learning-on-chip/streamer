#[macro_use]
extern crate log;

extern crate arguments;
extern crate configuration;
extern crate random;
extern crate sql;
extern crate sqlite;
extern crate streamer;
extern crate term;
extern crate time;

use configuration::format::TOML;
use log::LogLevel;
use streamer::{platform, schedule, system, traffic, workload};

pub use streamer::{Config, Error, Result};

pub type System = system::System<platform::Thermal,
                                 schedule::Compact,
                                 traffic::Fractal,
                                 workload::Random>;

pub type Increment = (system::Event, (platform::Profile, platform::Profile));

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
    ($message:expr) => (return Err(::streamer::Error::new($message)));
);

macro_rules! ok(
    ($result:expr) => (match $result {
        Ok(result) => result,
        Err(error) => raise!(error),
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
    start().unwrap_or_else(|error| fail(error));
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

    let config = ok!(TOML::open(some!(arguments.get::<String>("config"),
                                      "a configuration file is required")));

    let mut system = try!(construct_system(&config));
    let mut output = try!(output::new(&system, arguments.get::<String>("output")));

    let length = arguments.get::<f64>("length").unwrap_or(10.0);
    info!(target: "Streamer", "Simulating {} seconds...", length);

    let start = time::now();
    while let Some((event, (power, temperature))) = system.next() {
        let last = event.time > length;
        info!(target: "Streamer", "{} | {:2} queued", event,
              system.statistics().arrived - system.statistics().started);
        try!(output.next((event, (power, temperature))));
        if last {
            break;
        }
    }
    let elapsed = time::now() - start;

    info!(target: "Streamer", "Well done in {:.2} seconds.",
          elapsed.num_milliseconds() as f64 / 1000.0);

    Ok(())
}

fn construct_system(config: &Config) -> Result<System> {
    use streamer::platform::Platform;

    let source = {
        let seed = config.get::<i64>("seed").map(|&seed| seed as u64).unwrap_or(0);
        let seed = if seed > 0 { seed } else { time::now().to_timespec().sec as u64 };
        let seed = [0x12345678 & seed, 0x87654321 & seed];
        random::default().seed(seed)
    };

    macro_rules! branch(
        ($name:expr) => (config.branch($name).unwrap_or_else(|| Config::new()));
    );

    let platform = try!(platform::Thermal::new(&branch!("platform")));
    let schedule = try!(schedule::Compact::new(&branch!("schedule"), platform.elements()));
    let traffic = try!(traffic::Fractal::new(&branch!("traffic"), &source));
    let workload = try!(workload::Random::new(&branch!("workload"), &source));

    System::new(platform, schedule, traffic, workload)
}

fn help() -> ! {
    println!("{}", USAGE.trim());
    std::process::exit(0);
}

#[allow(unused_must_use)]
fn fail(error: Error) -> ! {
    use std::io::{stderr, Write};
    if let Some(mut output) = term::stderr() {
        output.fg(term::color::RED);
        output.write_all(format!("Error: {}.\n", error).as_bytes());
    }
    std::process::exit(1);
}
