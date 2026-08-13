use crate::profiler::zone::zone_id::ZoneId;
use std::time::Instant;

pub struct StackEntry {
    pub id: ZoneId,
    pub start: Option<Instant>,
}
