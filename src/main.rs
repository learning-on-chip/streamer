extern crate arguments;
extern crate sqlite;
extern crate toml;

use std::path::Path;

const USAGE: &'static str = "
Usage: streamer [options]

Options:
    --database <path>        SQLite3 database (required).
    --table <name>           Table containing area estimates (required).

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

    let _database = match arguments.get::<String>("database") {
        Some(ref database) => ok!(sqlite::open(&Path::new(database))),
        _ => raise!("a database filename is required"),
    };

    Ok(())
}

fn fail(error: Error) -> ! {
    use std::io::{stderr, Write};
    stderr().write_all(format!("Error: {}.\n", &*error).as_bytes()).unwrap();
    std::process::exit(1);
}

fn help() -> ! {
    println!("{}", USAGE.trim());
    std::process::exit(0);
}
