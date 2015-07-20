use fractal::Beta;
use sqlite::{Connection, State};
use std::collections::VecDeque;
use std::rc::Rc;

use config::Config;
use {Result, Source};

pub struct Traffic {
    model: Rc<Beta>,
    source: Source,
    steps: VecDeque<f64>,
}

impl Traffic {
    pub fn new(config: &Config, source: &Source) -> Result<Traffic> {
        let path = path!(config, "a workload pattern database");
        let backend = ok!(Connection::open(&path!(config, "a traffic database")));

        info!(target: "traffic", "Reading {:?}...", &path);
        let data = match config.get::<String>("query") {
            Some(ref query) => try!(read_interarrivals(&backend, query)),
            _ => raise!("an SQL query for reading the traffic data is required"),
        };
        let ncoarse = match (data.len() as f64).log2().floor() {
            ncoarse if ncoarse < 1.0 => raise!("there are not enough data"),
            ncoarse => ncoarse as usize,
        };

        info!(target: "traffic", "Read {} arrivals. Fitting the model...", data.len());
        Ok(Traffic {
            model: Rc::new(ok!(Beta::new(&data, ncoarse))),
            source: source.clone(),
            steps: VecDeque::new(),
        })
    }

    pub fn next(&mut self) -> Option<f64> {
        if self.steps.is_empty() {
            if let Err(error) = self.refill() {
                error!(target: "traffic", "Failed to refill the queue ({}).", error);
                return None;
            }
        }
        self.steps.pop_front()
    }

    fn refill(&mut self) -> Result<()> {
        info!(target: "traffic", "Refilling the queue...");
        self.steps.extend(&ok!(self.model.sample(&mut self.source)));
        info!(target: "traffic", "The queue contains {} interarrivals.", self.steps.len());
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
