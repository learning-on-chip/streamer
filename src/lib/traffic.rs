use fractal::Beta;
use sqlite::{Database, State};
use std::path::Path;

use Result;
use config;

pub struct Traffic {
    model: Beta,
}

impl Traffic {
    pub fn new<T: AsRef<Path>>(config: &config::Traffic, root: T) -> Result<Traffic> {
        let backend = ok!(Database::open(&path!(config.path, root.as_ref(),
                                                "a traffic database")));

        let data = match config.query {
            Some(ref query) => try!(read_interarrivals(&backend, query)),
            _ => raise!("an SQL query is required for the traffic database"),
        };

        let ncoarse = match (data.len() as f64).log2().floor() {
            ncoarse if ncoarse < 1.0 => raise!("there are not enought data"),
            ncoarse => ncoarse as usize,
        };

        Ok(Traffic { model: ok!(Beta::fit(&data, ncoarse)) })
    }
}

fn read_interarrivals(backend: &Database, query: &str) -> Result<Vec<f64>> {
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
    use config;
    use sqlite::Database;

    #[test]
    fn read_interarrivals() {
        let backend = Database::open("tests/fixtures/google.sqlite3").unwrap();
        let data = super::read_interarrivals(&backend,
            "SELECT 1e-6 * `time` FROM `job_events` \
             WHERE `time` IS NOT 0 AND `event type` is 0 \
             ORDER BY `time` ASC;"
        ).ok().unwrap();
        assert_eq!(data.len(), 19640);
    }
}
