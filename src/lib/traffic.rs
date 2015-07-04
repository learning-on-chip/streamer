use fractal::Beta;
use random::Source;
use sqlite::{Connection, State};
use std::path::Path;
use std::rc::Rc;

use Result;
use config;

pub struct Traffic {
    model: Rc<Beta>,
}

pub struct Queue<'l, S: Source + 'l> {
    model: Rc<Beta>,
    source: &'l mut S,
    time: f64,
    steps: Vec<f64>,
    position: usize,
}

impl Traffic {
    pub fn new<T: AsRef<Path>>(config: &config::Traffic, root: T) -> Result<Traffic> {
        let backend = ok!(Connection::open(&path!(config.path, root.as_ref(),
                                                  "a traffic database")));

        info!(target: "traffic", "Reading the database...");
        let data = match config.query {
            Some(ref query) => try!(read_interarrivals(&backend, query)),
            _ => raise!("an SQL query is required for the traffic database"),
        };
        info!(target: "traffic", "Read {} interarrivals.", data.len());

        let ncoarse = match (data.len() as f64).log2().floor() {
            ncoarse if ncoarse < 1.0 => raise!("there are not enought data"),
            ncoarse => ncoarse as usize,
        };

        info!(target: "traffic", "Fitting a model...");
        Ok(Traffic { model: Rc::new(ok!(Beta::fit(&data, ncoarse))) })
    }

    #[inline]
    pub fn iter<'l, S: Source + 'l>(&'l self, source: &'l mut S) -> Queue<'l, S> {
        Queue {
            model: self.model.clone(),
            source: source,
            time: 0.0,
            steps: vec![],
            position: 0,
        }
    }
}

impl<'l, S: Source> Queue<'l, S> {
    #[inline]
    fn is_empty(&self) -> bool {
        self.position >= self.steps.len()
    }

    #[inline]
    fn renew(&mut self) -> Result<()> {
        info!(target: "traffic", "Sampling the model...");
        self.steps = ok!(self.model.sample(self.source));
        info!(target: "traffic", "Sampled {} interarrivals.", self.steps.len());
        self.position = 0;
        Ok(())
    }
}

impl<'l, S: Source> Iterator for Queue<'l, S> {
    type Item = f64;

    fn next(&mut self) -> Option<f64> {
        if self.is_empty() {
            if let Err(error) = self.renew() {
                error!(target: "traffic", "Failed to sample the model ({}).", error);
                return None;
            }
        }
        self.time += self.steps[self.position];
        self.position += 1;
        Some(self.time)
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
        assert_eq!(data.len(), 19640);
    }
}
