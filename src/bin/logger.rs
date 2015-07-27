use log::{self, Log, LogLevel, LogMetadata, LogRecord};
use term;

pub struct Terminal(LogLevel);

impl Log for Terminal {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.0
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            let mut stdout = term::stdout();
            if record.metadata().level() < LogLevel::Info {
                stdout.as_mut().map(|stdout| stdout.fg(term::color::RED));
            } else {
                stdout.as_mut().map(|stdout| stdout.fg(term::color::GREEN));
            }
            print!("{:>12}", record.target());
            stdout.as_mut().map(|stdout| stdout.reset());
            println!(" {}", record.args());
        }
    }
}

pub fn setup(level: LogLevel) {
    let _ = log::set_logger(|max_log_level| {
        max_log_level.set(level.to_log_level_filter());
        Box::new(Terminal(level))
    });
}
