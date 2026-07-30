use std::any::Any;
use crate::profiler::meta_value::MetaValue;
use crate::profiler::zone::{ZoneId, ZoneKind};

pub struct FrameProfile {
    pub zones: Vec<ZoneEntry>,
    pub cpu_meta: Vec<CpuMetaEntry>,
    pub gpu_meta: Vec<GpuMetaEntry>,
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

pub struct GpuMetaEntry {
    pub name: &'static str,
    pub data: Box<dyn Any + Send + Sync>,
}

impl FrameProfile {
    pub fn empty() -> Self {
        Self {
            zones: Vec::new(),
            cpu_meta: Vec::new(),
            gpu_meta: Vec::new(),
        }
    }

    pub fn gpu_meta_for<T: 'static>(&self, name: &str) -> Option<&T> {
        self.gpu_meta
            .iter()
            .find(|entry| entry.name == name)
            .and_then(|entry| entry.data.downcast_ref::<T>())
    }
}
