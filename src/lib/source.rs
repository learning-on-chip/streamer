use sqlite::{Database, State};
use std::path::{Path, PathBuf};

use Result;
use config;

pub struct Source {
    pub names: Vec<String>,
    pub dynamic: Vec<f64>,
    pub leakage: Vec<f64>,
}

pub fn new(config: &config::Source, root: &Path) -> Result<Source> {
    let backend = {
        let mut path = match config.path {
            Some(ref path) => PathBuf::from(path),
            _ => raise!("a path to the database is required"),
        };
        if path.is_relative() {
            path = root.join(path);
        }
        if ::std::fs::metadata(&path).is_err() {
            raise!("the file {:?} does not exist", &path);
        }
        ok!(Database::open(&path))
    };

    let (names, leakage) = try!(read_names_and_leakage_power(&backend));
    let dynamic = try!(read_dynamic_power(&backend, &names));

    Ok(Source {
        names: names,
        dynamic: dynamic,
        leakage: leakage,
    })
}

fn read_names_and_leakage_power(backend: &Database) -> Result<(Vec<String>, Vec<f64>)> {
    let mut names = vec![];
    let mut data = vec![];
    let mut statement = ok!(backend.prepare(
        r#"SELECT `name`, `value` from `static` where `name` LIKE "%_leakage_power";"#
    ));
    while State::Row == ok!(statement.step()) {
        names.push({
            let name = ok!(statement.column::<String>(0));
            String::from(&name[..name.find('_').unwrap()])
        });
        data.push(ok!(statement.column::<f64>(1)));
    }
    Ok((names, data))
}

fn read_dynamic_power(backend: &Database, names: &[String]) -> Result<Vec<f64>> {
    let count = names.len();
    let mut data = Vec::with_capacity(count);
    let fields = {
        let mut buffer = String::new();
        for name in names.iter() {
            if !buffer.is_empty() {
                buffer.push_str(", ");
            }
            buffer.push_str(&format!("`{}_dynamic_power`", name));
        }
        buffer
    };
    let mut statement = ok!(backend.prepare(&format!(
        "SELECT {} FROM `dynamic` ORDER BY `time` ASC;", &fields,
    )));
    while State::Row == ok!(statement.step()) {
        for i in 0..count {
            data.push(ok!(statement.column::<f64>(i)));
        }
    }
    Ok(data)
}
