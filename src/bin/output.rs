use sqlite::{Connection, State, Statement};
use std::mem;
use std::path::Path;
use streamer::{Increment, Profile, Result, System};

const QUERY_CREATE: &'static str = "
    CREATE TABLE IF NOT EXISTS `dynamic` (
        `time` REAL NOT NULL,
        `component_id` INTEGER NOT NULL,
        `power` REAL NOT NULL,
        `temperature` REAL NOT NULL
    );
    DELETE FROM `dynamic`;
";

const QUERY_INSERT: &'static str = "
    INSERT INTO `dynamic` (`time`, `component_id`, `power`, `temperature`) VALUES (?, ?, ?, ?)
";

const QUERY_INSERT_MORE: &'static str = "
    , (?, ?, ?, ?)
";

pub trait Output {
    fn next(&mut self, Increment) -> Result<()>;
}

pub struct Database<'l> {
    #[allow(dead_code)]
    connection: Connection<'l>,
    statement: Statement<'static>,
}

pub struct Terminal;

impl<'l> Database<'l> {
    pub fn new<T: AsRef<Path>>(system: &System, path: T) -> Result<Database<'l>> {
        let units = system.platform.units;
        let connection = ok!(Connection::open(path));
        ok!(connection.execute(QUERY_CREATE));
        let statement = {
            let mut statement = QUERY_INSERT.to_string();
            for _ in 1..units {
                statement.push_str(QUERY_INSERT_MORE);
            }
            unsafe { mem::transmute(ok!(connection.prepare(&statement))) }
        };
        Ok(Database { connection: connection, statement: statement })
    }
}

impl<'l> Output for Database<'l> {
    fn next(&mut self, (_, power, temperature): Increment) -> Result<()> {
        let Profile { units, steps, time, time_step, data: power } = power;
        let Profile { data: temperature, .. } = temperature;
        let statement = &mut self.statement;
        for i in 0..steps {
            let time = time + (i as f64) * time_step;
            ok!(statement.reset());
            let mut k = 1;
            for j in 0..units {
                ok!(statement.bind(k + 0, time));
                ok!(statement.bind(k + 1, j as i64));
                ok!(statement.bind(k + 2, power[i * units + j]));
                ok!(statement.bind(k + 3, temperature[i * units + j]));
                k += 4;
            }
            if State::Done != ok!(statement.step()) {
                raise!("failed to write into the database");
            }
        }
        Ok(())
    }
}

impl Output for Terminal {
    fn next(&mut self, (event, power, _): Increment) -> Result<()> {
        if power.steps > 0 {
            println!("{} - {} samples", event, power.steps);
        } else {
            println!("{}", event);
        }
        Ok(())
    }
}

pub fn new<T: AsRef<Path>>(system: &System, output: Option<T>) -> Result<Box<Output>> {
    Ok(match output {
        Some(output) => Box::new(try!(Database::new(system, output))),
        _ => Box::new(Terminal),
    })
}
