use crate::profiler::zone::ZoneId;
use std::time::Instant;

pub struct StackEntry {
    pub id: ZoneId,
    pub start: Option<Instant>,
}
