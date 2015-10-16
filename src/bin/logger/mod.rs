use log::{self, LogLevel};

mod terminal;

use self::terminal::Terminal;

pub fn setup(level: LogLevel) {
    let _ = log::set_logger(|max_log_level| {
        max_log_level.set(level.to_log_level_filter());
        Box::new(Terminal(level))
    });
}
