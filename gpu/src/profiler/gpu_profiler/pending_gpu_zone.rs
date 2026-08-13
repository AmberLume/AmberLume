use crate::profiler::zone::zone_id::ZoneId;

pub struct PendingGpuZone {
    pub zone_id: ZoneId,
    pub parent: Option<ZoneId>,
    pub slot: u32,
}
