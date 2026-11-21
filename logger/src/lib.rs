mod log_level;
mod log_output;
mod log_record;
mod logger;
mod macros;

pub use log_level::LogLevel;
pub use log_output::LogOutput;
pub use log_record::LogRecord;
pub use logger::Logger;

pub const VALIDATION: bool = cfg!(any(debug_assertions, feature = "validation"));
