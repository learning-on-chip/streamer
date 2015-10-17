use fractal::Beta;
use sqlite::{Connection, State};
use std::collections::VecDeque;

use {Config, Result, Source};

pub struct Traffic {
    time: f64,
    model: Beta,
    source: Source,
    arrivals: VecDeque<f64>,
}

macro_rules! refill(
    ($this:ident) => (
        if $this.arrivals.is_empty() {
            if let Err(error) = $this.refill() {
                error!(target: "Traffic", "Failed to refill the queue ({}).", error);
                return None;
            }
        }
    );
);

impl Traffic {
    pub fn new(config: &Config, source: &Source) -> Result<Traffic> {
        let path = path!(config, "a traffic-pattern database is required");
        let backend = ok!(Connection::open(&path));

        info!(target: "Traffic", "Reading {:?}...", &path);
        let data = try!(read_interarrivals(&backend));
        let ncoarse = match (data.len() as f64).log2().floor() {
            ncoarse if ncoarse < 1.0 => raise!("there are not enough data"),
            ncoarse => ncoarse as usize,
        };

        info!(target: "Traffic", "Read {} arrivals. Fitting the model...", data.len());
        Ok(Traffic {
            time: 0.0,
            model: ok!(Beta::new(&data, ncoarse)),
            source: source.clone(),
            arrivals: VecDeque::new(),
        })
    }

    pub fn next(&mut self) -> Option<f64> {
        refill!(self);
        self.arrivals.pop_front()
    }

    pub fn peek(&mut self) -> Option<&f64> {
        refill!(self);
        self.arrivals.get(0)
    }

    fn refill(&mut self) -> Result<()> {
        info!(target: "Traffic", "Refilling the queue...");
        for step in ok!(self.model.sample(&mut self.source)) {
            self.time += step;
            self.arrivals.push_back(self.time);
        }
        info!(target: "Traffic", "The queue contains {} arrivals.", self.arrivals.len());
        Ok(())
    }
}

fn read_interarrivals(backend: &Connection) -> Result<Vec<f64>> {
    use sql::prelude::*;

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
    use sqlite::Connection;

    #[test]
    fn read_interarrivals() {
        let backend = Connection::open("tests/fixtures/google.sqlite3").unwrap();
        let data = super::read_interarrivals(&backend).unwrap();
        assert_eq!(data.len(), 668088);
    }
}
