use log::{Log, LogLevel, LogMetadata, LogRecord};
use term;

pub struct Terminal(pub LogLevel);

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
