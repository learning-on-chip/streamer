extern crate arguments;
extern crate configuration;
extern crate term;

#[macro_use] extern crate log;
#[macro_use] extern crate streamer;

use configuration::format::TOML;
use log::LogLevel;
use streamer::{Result, platform, schedule, traffic, workload};
use streamer::output::{self, Output};
use streamer::system::{self, Event};

mod logger;

use logger::Logger;

type System = system::System<traffic::Fractal,
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

#[allow(unused_must_use)]
fn main() {
    if let Err(error) = start() {
        use std::io::Write;
        if let Some(mut output) = term::stderr() {
            output.fg(term::color::RED);
            output.write_fmt(format_args!("Error: {}.\n", error));
            output.reset();
        }
        std::process::exit(1);
    }
}

fn start() -> Result<()> {
    let arguments = ok!(arguments::parse(std::env::args()));
    if arguments.get::<bool>("help").unwrap_or(false) {
        println!("{}", USAGE.trim());
        return Ok(());
    }
    if arguments.get::<bool>("verbose").unwrap_or(false) {
        Logger::install(LogLevel::Info);
    } else {
        Logger::install(LogLevel::Warn);
    }

    let mut system = {
        let config = ok!(TOML::open(some!(arguments.get::<String>("config"),
                                          "a configuration file is required")));
        macro_rules! branch(($name:expr) => (config.branch($name).as_ref().unwrap_or(&config)));
        let source = streamer::source(&config);
        let traffic = try!(traffic::Fractal::new(branch!("traffic"), &source));
        let workload = try!(workload::Random::new(branch!("workload"), &source));
        let platform = try!(platform::Thermal::new(branch!("platform")));
        let schedule = try!(schedule::Impartial::new(branch!("schedule"), &platform));
        try!(System::new(traffic, workload, platform, schedule))
    };
    let mut output = match arguments.get::<String>("output") {
        Some(path) => Some(try!(output::Thermal::new(system.platform(), path))),
        _ => None,
    };

    let length = arguments.get::<f64>("length").unwrap_or(10.0);
    info!(target: "Streamer", "Synthesizing {} seconds...", length);
    while let Some((event, data)) = try!(system.next()) {
        display(&system, &event);
        if let Some(ref mut output) = output {
            try!(output.next(&event, &data));
        }
        if event.time > length {
            break;
        }
    }
    info!(target: "Streamer", "Well done.");

    Ok(())
}

fn display(system: &System, event: &Event) {
    use streamer::system::EventKind;

    let (job, kind) = match &event.kind {
        &EventKind::Arrived(ref job) => (job, "arrived"),
        &EventKind::Started(ref job) => (job, "started"),
        &EventKind::Finished(ref job) => (job, "finished"),
    };
    info!(target: "Streamer",
          "{:8.2} s | job #{:4} ( {:20} | {:2} units | {:6.2} s ) {:8} | {:2} queued",
          event.time, job.id, job.name, job.units, job.duration(), kind,
          system.history().arrived - system.history().started);
}
