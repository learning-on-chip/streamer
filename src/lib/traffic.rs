use fractal::Beta;
use sqlite::{Connection, State};
use std::collections::VecDeque;

use config::Config;
use {Result, Source};

pub struct Traffic {
    time: f64,
    model: Beta,
    source: Source,
    arrivals: VecDeque<f64>,
}

impl Traffic {
    pub fn new(config: &Config, source: &Source) -> Result<Traffic> {
        let path = path!(config, "a workload pattern database");
        let backend = ok!(Connection::open(&path!(config, "a traffic database")));

        info!(target: "Traffic", "Reading {:?}...", &path);
        let data = {
            let query = some!(config.get::<String>("query"),
                              "an SQL query for reading the traffic data");
            try!(read_interarrivals(&backend, query))
        };
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
        if let Err(error) = self.refill() {
            error!(target: "Traffic", "Failed to refill the queue ({}).", error);
            return None;
        }
        self.arrivals.pop_front()
    }

    pub fn peek(&mut self) -> Option<&f64> {
        if let Err(error) = self.refill() {
            error!(target: "Traffic", "Failed to refill the queue ({}).", error);
            return None;
        }
        self.arrivals.get(0)
    }

    fn refill(&mut self) -> Result<()> {
        if self.arrivals.is_empty() {
            info!(target: "Traffic", "Refilling the queue...");
            for step in ok!(self.model.sample(&mut self.source)) {
                self.time += step;
                self.arrivals.push_back(self.time);
            }
            info!(target: "Traffic", "The queue contains {} arrivals.", self.arrivals.len());
        }
        Ok(())
    }
}

fn read_interarrivals(backend: &Connection, query: &str) -> Result<Vec<f64>> {
    let mut statement = ok!(backend.prepare(query));

    let mut data = Vec::new();
    let mut last_time = {
        if State::Row != ok!(statement.step()) {
            return Ok(data);
        }
        ok!(statement.read::<f64>(0))
    };
    while State::Row == ok!(statement.step()) {
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
        let data = super::read_interarrivals(&backend, "
            SELECT 1e-6 * `time` FROM `job_events`
            WHERE `time` IS NOT 0 AND `event type` IS 0
            ORDER BY `time` ASC;
        ").ok().unwrap();
        assert_eq!(data.len(), 667926);
    }
}
