use logger::LogLevel;
use logger::LogOutput;
use logger::LogRecord;
use std::io::Write;
use std::io::stdout;
use std::thread;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub struct ConsoleLogOutput;

impl ConsoleLogOutput {
    fn level_name(level: LogLevel) -> &'static str {
        match level {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }
}

impl LogOutput for ConsoleLogOutput {
    fn write(&self, record: &LogRecord) {
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before 1970");
        let seconds = since_epoch.as_secs() % 86_400;

        let thread = thread::current();
        let thread_name = thread.name().unwrap_or("unnamed");

        let _ = writeln!(
            stdout().lock(),
            "{:02}:{:02}:{:02}.{:06} {:>5} {} {}: {}",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60,
            since_epoch.subsec_micros(),
            Self::level_name(record.level),
            thread_name,
            record.target,
            record.message,
        );
    }
}
