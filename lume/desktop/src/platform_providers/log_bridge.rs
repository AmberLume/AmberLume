use anyhow::{anyhow, Result};
use log::{Level, LevelFilter, Log, Metadata, Record};
use logger::{LogLevel, LogRecord, Logger};

pub struct LogBridge;

impl LogBridge {
    pub fn install() -> Result<()> {
        let level = Logger::level().ok_or_else(|| anyhow!("log bridge install: logger is not initialized"))?;

        log::set_logger(&LogBridge).map_err(|error| anyhow!("log bridge install: {error}"))?;
        log::set_max_level(Self::level_filter(level));

        Ok(())
    }

    fn level_filter(level: LogLevel) -> LevelFilter {
        match level {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Fatal => LevelFilter::Off,
        }
    }

    fn level(level: Level) -> LogLevel {
        match level {
            Level::Error => LogLevel::Error,
            Level::Warn => LogLevel::Warn,
            Level::Info => LogLevel::Info,
            Level::Debug => LogLevel::Debug,
            Level::Trace => LogLevel::Trace,
        }
    }
}

impl Log for LogBridge {
    fn enabled(&self, metadata: &Metadata) -> bool {
        Logger::enabled(Self::level(metadata.level()))
    }

    fn log(&self, record: &Record) {
        Logger::log(&LogRecord {
            level: Self::level(record.level()),
            target: record.target(),
            message: *record.args(),
            file: record.file(),
            line: record.line(),
        });
    }

    fn flush(&self) {}
}
