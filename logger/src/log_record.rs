use crate::log_level::LogLevel;
use std::fmt::Arguments;

pub struct LogRecord<'a> {
    pub level: LogLevel,
    pub target: &'a str,
    pub message: Arguments<'a>,
    pub file: Option<&'a str>,
    pub line: Option<u32>,
}
