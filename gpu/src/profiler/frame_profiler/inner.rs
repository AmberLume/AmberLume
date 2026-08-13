use std::collections::HashMap;
use index_allocator::FrameIndex;
use crate::profiler::frame_profile::cpu_meta_entry::CpuMetaEntry;
use crate::profiler::frame_profile::zone_entry::ZoneEntry;
use crate::profiler::gpu_profiler::gpu_profiler::GpuProfiler;
use crate::profiler::stack_entry::StackEntry;
use crate::profiler::zone::zone_id::ZoneId;
use crate::profiler::zone_slot::ZoneSlot;

pub(super) struct Inner {
    pub(super) zone_slots: Vec<ZoneSlot>,
    pub(super) zone_index_by_name: HashMap<&'static str, ZoneId>,

    pub(super) stack: Vec<StackEntry>,
    pub(super) events: Vec<ZoneEntry>,

    pub(super) cpu_meta: Vec<CpuMetaEntry>,
    pub(super) cpu_meta_index_by_name: HashMap<&'static str, usize>,

    pub(super) current_frame_index: FrameIndex,

    pub(super) gpu: GpuProfiler,
}

impl Inner {
    pub(super) fn get_or_create_zone(&mut self, name: &'static str) -> ZoneId {
        if let Some(&id) = self.zone_index_by_name.get(name) {
            return id;
        }

        let id = ZoneId::new(self.zone_slots.len() as u32);
        self.zone_slots.push(ZoneSlot { name });
        self.zone_index_by_name.insert(name, id);

        id
    }
}
