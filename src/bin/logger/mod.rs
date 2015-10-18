use log::{self, LogLevel};

mod terminal;

use self::terminal::Terminal;

#[allow(unused_must_use)]
pub fn setup(level: LogLevel) {
    log::set_logger(|max_log_level| {
        max_log_level.set(level.to_log_level_filter());
        Box::new(Terminal(level))
    });
}
