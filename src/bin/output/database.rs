use sqlite::{Connection, Statement, State};
use std::mem;
use std::path::Path;

use output::Output;
use streamer::platform::{Platform, Profile};
use {Data, Event, Result, System};

pub struct Database {
    #[allow(dead_code)]
    connection: Connection,
    statement: Statement<'static>,
}

impl Database {
    pub fn new<T: AsRef<Path>>(system: &System, path: T) -> Result<Database> {
        use sql::prelude::*;

        let connection = ok!(Connection::open(path));

        ok!(connection.execute({
            ok!(create_table("dynamic").if_not_exists().columns(&[
                "time".float().not_null(), "component_id".integer().not_null(),
                "power".float().not_null(), "temperature".float().not_null(),
            ]).compile())
        }));

        ok!(connection.execute(ok!(delete_from("dynamic").compile())));

        let statement = {
            let units = system.platform().elements().len();
            let statement = ok!(connection.prepare({
                ok!(insert_into("dynamic").columns(&[
                    "time", "component_id", "power", "temperature",
                ]).batch(units).compile())
            }));
            unsafe { mem::transmute(statement) }
        };

        Ok(Database { connection: connection, statement: statement })
    }
}

impl Output for Database {
    fn next(&mut self, _: &Event, &(ref power, ref temperature): &Data) -> Result<()> {
        let &Profile { units, steps, time, time_step, data: ref power } = power;
        let &Profile { data: ref temperature, .. } = temperature;
        let statement = &mut self.statement;
        for i in 0..steps {
            let time = time + (i as f64) * time_step;
            ok!(statement.reset());
            let mut k = 0;
            for j in 0..units {
                ok!(statement.bind(k + 1, time));
                ok!(statement.bind(k + 2, j as i64));
                ok!(statement.bind(k + 3, power[i * units + j]));
                ok!(statement.bind(k + 4, temperature[i * units + j]));
                k += 4;
            }
            if State::Done != ok!(statement.next()) {
                raise!("failed to write into the database");
            }
        }
        Ok(())
    }
}
