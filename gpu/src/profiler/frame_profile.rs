use crate::profiler::meta_value::MetaValue;
use crate::profiler::zone::{ZoneId, ZoneKind};

pub struct FrameProfile {
    pub zones: Vec<ZoneEntry>,
    pub cpu_meta: Vec<CpuMetaEntry>,
}

pub struct ZoneEntry {
    pub id: ZoneId,
    pub parent: Option<ZoneId>,
    pub name: &'static str,
    pub kind: ZoneKind,
    pub duration_ns: u64,
}

pub struct CpuMetaEntry {
    pub name: &'static str,
    pub value: MetaValue,
}


impl FrameProfile {
    pub fn empty() -> Self {
        Self {
            zones: Vec::new(),
            cpu_meta: Vec::new(),
        }
    }
}
