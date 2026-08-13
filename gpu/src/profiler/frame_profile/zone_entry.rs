use crate::profiler::zone::zone_id::ZoneId;
use crate::profiler::zone::zone_kind::ZoneKind;

pub struct ZoneEntry {
    pub id: ZoneId,
    pub parent: Option<ZoneId>,
    pub name: &'static str,
    pub kind: ZoneKind,
    pub duration_ns: u64,
}
