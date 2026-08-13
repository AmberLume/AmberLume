use crate::profiler::zone::zone_id::ZoneId;

pub struct ResolvedGpuZone {
    pub zone_id: ZoneId,
    pub parent: Option<ZoneId>,
    pub duration_ns: u64,
}
