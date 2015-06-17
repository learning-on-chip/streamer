use sqlite::Database;

pub trait Source {
}

impl<'l> Source for Database<'l> {
}
