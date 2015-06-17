extern crate arguments;
extern crate sqlite;
extern crate toml;

use std::path::Path;

const USAGE: &'static str = "
Usage: streamer [options] <database>...

Options:
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

    let mut databases = vec![];
    for ref path in arguments.orphans.iter() {
        let path = Path::new(path);
        if std::fs::metadata(path).is_err() {
            raise!("the database {:?} does not exist", path);
        }
        databases.push(ok!(sqlite::open(path)));
    }
    if databases.is_empty() {
        raise!("at least one database is required");
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
