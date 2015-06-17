use sqlite::Database;
use std::path::{Path, PathBuf};

use Result;
use config;

pub struct SQLite<'l> {
    backend: Database<'l>,
}

impl<'l> SQLite<'l> {
    pub fn new(config: &config::Source, root: &Path) -> Result<SQLite<'l>> {
        let mut path = match config.path {
            Some(ref path) => PathBuf::from(path),
            _ => raise!("the path to the database is required"),
        };
        if path.is_relative() {
            path = root.join(path);
        }
        if ::std::fs::metadata(&path).is_err() {
            raise!("the database file {:?} does not exist", &path);
        }
        Ok(SQLite { backend: ok!(Database::open(&path)) })
    }
}

impl<'l> super::Source for SQLite<'l> {
}
