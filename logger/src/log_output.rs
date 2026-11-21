use crate::log_record::LogRecord;

pub trait LogOutput: Send + Sync {
    fn write(&self, record: &LogRecord);
}
