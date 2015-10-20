extern crate arguments;
extern crate configuration;
extern crate random;
extern crate sql;
extern crate sqlite;
extern crate term;
extern crate time;

#[macro_use]
extern crate log;

#[macro_use]
extern crate streamer;

use configuration::format::TOML;
use log::LogLevel;
use streamer::{platform, schedule, system, traffic, workload};

pub use streamer::{Config, Error, Result};

pub type Data = (platform::Profile, platform::Profile);
pub type Event = system::Event;
pub type System = system::System<traffic::Fractal,
                                 workload::Random,
                                 platform::Thermal,
                                 schedule::Impartial>;

const USAGE: &'static str = "
Usage: streamer [options]

Options:
    --config <path>          Configuration file (required).
    --length <time>          Time span to synthesize in seconds [default: 10].
    --output <path>          Output file for power and temperature profiles.

    --verbose                Display progress information.
    --help                   Display this message.
";

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
    info!(target: "Streamer", "Synthesizing {} seconds...", length);
    let start = time::now();

    while let Some((event, data)) = try!(system.next()) {
        display(&system, &event);
        try!(output.next(&event, &data));
        if event.time > length {
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

    macro_rules! branch(($name:expr) => (config.branch($name).as_ref().unwrap_or(config)));

    let traffic = try!(traffic::Fractal::new(branch!("traffic"), &source));
    let workload = try!(workload::Random::new(branch!("workload"), &source));
    let platform = try!(platform::Thermal::new(branch!("platform")));
    let schedule = try!(schedule::Impartial::new(branch!("schedule"), platform.elements()));

    System::new(traffic, workload, platform, schedule)
}

fn display(system: &System, event: &Event) {
    use streamer::system::EventKind;

    let (job, kind) = match &event.kind {
        &EventKind::Arrived(ref job) => (job, "arrived"),
        &EventKind::Started(ref job) => (job, "started"),
        &EventKind::Finished(ref job) => (job, "finished"),
    };
    info!(target: "Streamer",
          "{:7.2} s | job #{:3} ( {:20} | {:2} units | {:6.2} s ) {:8} | {:2} queued",
          event.time, job.id, job.name, job.units, job.duration(), kind,
          system.history().arrived - system.history().started);
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
