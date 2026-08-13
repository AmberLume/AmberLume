use crate::profiler::frame_profile::cpu_meta_entry::CpuMetaEntry;
use crate::profiler::frame_profile::zone_entry::ZoneEntry;

pub struct FrameProfile {
    pub zones: Vec<ZoneEntry>,
    pub cpu_meta: Vec<CpuMetaEntry>,
}

impl FrameProfile {
    pub fn empty() -> Self {
        Self {
            zones: Vec::new(),
            cpu_meta: Vec::new(),
        }
    }
}
