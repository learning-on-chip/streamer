//! Traffic modeling.

use std::path::Path;

use Result;

mod fractal;

pub use self::fractal::Fractal;

/// A traffic model.
pub trait Traffic {
    /// Return the next arrival time.
    fn next(&mut self) -> Result<Option<f64>>;

    /// Peek at the next arrival time.
    fn peek(&mut self) -> Result<Option<&f64>>;
}

fn read_interarrivals<T: AsRef<Path>>(path: T) -> Result<Vec<f64>> {
    use sql::prelude::*;
    use sqlite::{Connection, State};

    let backend = ok!(Connection::open(path));

    let statement = select_from("arrivals").column("time").order_by(column("time").ascend());
    let mut statement = ok!(backend.prepare(ok!(statement.compile())));

    let mut data = Vec::new();
    let mut last_time = {
        if let State::Done = ok!(statement.next()) {
            return Ok(data);
        }
        ok!(statement.read::<f64>(0))
    };
    while let State::Row = ok!(statement.next()) {
        let time = ok!(statement.read::<f64>(0));
        data.push(time - last_time);
        last_time = time;
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    #[test]
    fn read_interarrivals() {
        let data = super::read_interarrivals("tests/fixtures/google.sqlite3").unwrap();
        assert_eq!(data.len(), 668088);
    }
}
