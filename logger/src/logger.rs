use crate::log_level::LogLevel;
use crate::log_output::LogOutput;
use crate::log_record::LogRecord;
use anyhow::{anyhow, Result};
use std::sync::OnceLock;

static LOGGER: OnceLock<Logger> = OnceLock::new();

pub struct Logger {
    level: LogLevel,
    outputs: Vec<Box<dyn LogOutput>>,
}

impl Logger {
    pub fn initialize(level: LogLevel, outputs: Vec<Box<dyn LogOutput>>) -> Result<()> {
        LOGGER
            .set(Self { level, outputs })
            .map_err(|_| anyhow!("logger is already initialized"))
    }

    pub fn level() -> Option<LogLevel> {
        LOGGER.get().map(|logger| logger.level)
    }

    pub fn enabled(level: LogLevel) -> bool {
        LOGGER.get().is_some_and(|logger| level >= logger.level)
    }

    pub fn log(record: &LogRecord) {
        let Some(logger) = LOGGER.get() else {
            return;
        };

        if record.level < logger.level {
            return;
        }

        for output in &logger.outputs {
            output.write(record);
        }
    }
}
