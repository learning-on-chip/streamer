use arguments::Arguments;
use sqlite::{Connection, State, Statement};
use std::mem;
use std::path::Path;
use streamer::{Increment, Profile, Result};

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
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Database<'l>> {
        let connection = ok!(Connection::open(path));
        ok!(connection.execute(QUERY_CREATE));
        let statement = unsafe { mem::transmute(ok!(connection.prepare(QUERY_INSERT))) };
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
            for j in 0..units {
                ok!(statement.reset());
                ok!(statement.bind(1, time));
                ok!(statement.bind(2, j as i64));
                ok!(statement.bind(3, power[i * units + j]));
                ok!(statement.bind(4, temperature[i * units + j]));
                if State::Done != ok!(statement.step()) {
                    raise!("failed to write into the database");
                }
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

pub fn new(arguments: &Arguments) -> Result<Box<Output>> {
    Ok(match arguments.get::<String>("output") {
        Some(ref output) => Box::new(try!(Database::new(output))),
        _ => Box::new(Terminal),
    })
}
