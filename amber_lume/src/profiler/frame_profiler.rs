use crate::ids::FrameIndex;
use crate::profiler::frame_profile::{CpuMetaEntry, FrameProfile, ZoneEntry};
use crate::profiler::gpu_profiler::{GpuProfiler, PendingGpuZone};
use crate::profiler::meta_value::MetaValue;
use crate::profiler::stack_entry::StackEntry;
use crate::profiler::zone::{ZoneId, ZoneKind};
use crate::profiler::zone_slot::ZoneSlot;
use crate::render::device::device_context::DeviceContext;
use crate::render::factories::resource_factories::ResourceFactories;
use anyhow::Result;
use arc_swap::ArcSwap;
use ash::vk::CommandBuffer;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::mem::take;
use std::sync::Arc;
use std::time::Instant;

pub struct CpuZoneGuard<'a> {
    profiler: &'a FrameProfiler,
    id: ZoneId,
    parent: Option<ZoneId>,
}

impl<'a> Drop for CpuZoneGuard<'a> {
    fn drop(&mut self) {
        let mut inner = self.profiler.inner.lock();

        let entry = inner.stack.pop().expect("zone stack underflow");
        debug_assert_eq!(entry.id, self.id);

        let start = entry.start.expect("CpuZoneGuard end without start");
        let duration_ns = start.elapsed().as_nanos() as u64;
        let name = inner.zone_slots[self.id.index()].name;

        inner.events.push(ZoneEntry {
            id: self.id,
            parent: self.parent,
            name,
            kind: ZoneKind::Cpu,
            duration_ns,
        });
    }
}

pub struct GpuZoneGuard<'a> {
    profiler: &'a FrameProfiler,
    cmd: CommandBuffer,
    zone_id: ZoneId,
    parent: Option<ZoneId>,
    frame_index: FrameIndex,
    slot: u32,
}

impl<'a> Drop for GpuZoneGuard<'a> {
    fn drop(&mut self) {
        let mut inner = self.profiler.inner.lock();

        inner.gpu.write_end(self.cmd, self.frame_index, self.slot);

        let entry = inner.stack.pop().expect("zone stack underflow");
        debug_assert_eq!(entry.id, self.zone_id);

        inner.gpu.record_pending(
            self.frame_index,
            PendingGpuZone {
                zone_id: self.zone_id,
                parent: self.parent,
                slot: self.slot,
            },
        );
    }
}

struct Inner {
    zone_slots: Vec<ZoneSlot>,
    zone_index_by_name: HashMap<&'static str, ZoneId>,

    stack: Vec<StackEntry>,
    events: Vec<ZoneEntry>,

    cpu_meta: Vec<CpuMetaEntry>,
    cpu_meta_index_by_name: HashMap<&'static str, usize>,

    current_frame_index: FrameIndex,

    gpu: GpuProfiler,
}

impl Inner {
    fn get_or_create_zone(&mut self, name: &'static str) -> ZoneId {
        if let Some(&id) = self.zone_index_by_name.get(name) {
            return id;
        }

        let id = ZoneId::new(self.zone_slots.len() as u32);
        self.zone_slots.push(ZoneSlot { name });
        self.zone_index_by_name.insert(name, id);

        id
    }
}

pub struct FrameProfiler {
    inner: Mutex<Inner>,
    last_profile: ArcSwap<FrameProfile>,
}

impl FrameProfiler {
    pub fn new(
        device_context: &DeviceContext,
        resource_factories: &ResourceFactories,
        frames_in_flight: u32,
        max_gpu_zones: u32,
    ) -> Result<Self> {
        let gpu = GpuProfiler::new(
            device_context,
            resource_factories,
            frames_in_flight,
            max_gpu_zones,
        )?;

        Ok(Self {
            inner: Mutex::new(Inner {
                zone_slots: Vec::new(),
                zone_index_by_name: HashMap::new(),

                stack: Vec::new(),
                events: Vec::new(),

                cpu_meta: Vec::new(),
                cpu_meta_index_by_name: HashMap::new(),

                current_frame_index: FrameIndex::ZERO,

                gpu,
            }),
            last_profile: ArcSwap::new(Arc::new(FrameProfile::empty())),
        })
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        self.inner.into_inner().gpu.destroy(resource_factories)
    }

    pub fn begin_frame(&self, frame_index: FrameIndex) {
        let mut inner = self.inner.lock();

        inner.events.clear();
        inner.cpu_meta.clear();
        inner.cpu_meta_index_by_name.clear();
        inner.current_frame_index = frame_index;

        let resolved = inner.gpu.collect_previous(frame_index);
        for zone in resolved {
            let name = inner.zone_slots[zone.zone_id.index()].name;
            inner.events.push(ZoneEntry {
                id: zone.zone_id,
                parent: zone.parent,
                name,
                kind: ZoneKind::Gpu,
                duration_ns: zone.duration_ns,
            });
        }
    }

    pub fn extract_queries(&self, cmd: CommandBuffer, frame_index: FrameIndex) {
        self.inner.lock().gpu.extract(cmd, frame_index);
    }

    pub fn cpu_zone(&self, name: &'static str) -> CpuZoneGuard<'_> {
        let mut inner = self.inner.lock();

        let id = inner.get_or_create_zone(name);
        let parent = inner.stack.last().map(|entry| entry.id);

        inner.stack.push(StackEntry {
            id,
            start: Some(Instant::now()),
        });

        CpuZoneGuard {
            profiler: self,
            id,
            parent,
        }
    }

    pub fn gpu_zone(&self, cmd: CommandBuffer, name: &'static str) -> GpuZoneGuard<'_> {
        let mut inner = self.inner.lock();

        let id = inner.get_or_create_zone(name);
        let parent = inner.stack.last().map(|entry| entry.id);
        inner.stack.push(StackEntry { id, start: None });

        let frame_index = inner.current_frame_index;
        let slot = inner.gpu.allocate_slot(id).unwrap();
        inner.gpu.write_start(cmd, frame_index, slot);

        GpuZoneGuard {
            profiler: self,
            cmd,
            zone_id: id,
            parent,
            frame_index,
            slot,
        }
    }

    pub fn cpu_meta(&self, name: &'static str, value: MetaValue) {
        let mut inner = self.inner.lock();

        match inner.cpu_meta_index_by_name.get(name).copied() {
            Some(index) => inner.cpu_meta[index].value = value,
            None => {
                let index = inner.cpu_meta.len();
                inner.cpu_meta_index_by_name.insert(name, index);
                inner.cpu_meta.push(CpuMetaEntry { name, value });
            }
        }
    }

    pub fn end_frame(&self) {
        let profile = {
            let mut inner = self.inner.lock();

            debug_assert!(inner.stack.is_empty(), "frame ended with open zones");

            let profile = FrameProfile {
                zones: take(&mut inner.events),
                cpu_meta: take(&mut inner.cpu_meta),
                gpu_meta: Vec::new(),
            };

            inner.cpu_meta_index_by_name.clear();

            profile
        };

        self.last_profile.store(Arc::new(profile));
    }

    pub fn last_profile(&self) -> Arc<FrameProfile> {
        self.last_profile.load_full()
    }
}
