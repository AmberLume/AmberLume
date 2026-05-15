use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::take;
use std::sync::Arc;
use std::time::Instant;

use crate::profiler::frame_profile::{CpuMetaEntry, FrameProfile, ZoneEntry};
use crate::profiler::meta_value::MetaValue;
use crate::profiler::zone::{ZoneId, ZoneKind};

pub struct FrameProfiler {
    inner: RefCell<Inner>,
    last_profile: RefCell<Arc<FrameProfile>>,
}

struct Inner {
    zone_slots: Vec<ZoneSlot>,
    zone_index_by_name: HashMap<&'static str, ZoneId>,

    stack: Vec<StackEntry>,
    events: Vec<ZoneEntry>,

    cpu_meta: Vec<CpuMetaEntry>,
    cpu_meta_index_by_name: HashMap<&'static str, usize>,
}

struct ZoneSlot {
    name: &'static str,
}

struct StackEntry {
    id: ZoneId,
    start: Instant,
}

impl FrameProfiler {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(Inner {
                zone_slots: Vec::new(),
                zone_index_by_name: HashMap::new(),

                stack: Vec::new(),
                events: Vec::new(),

                cpu_meta: Vec::new(),
                cpu_meta_index_by_name: HashMap::new(),
            }),
            last_profile: RefCell::new(Arc::new(FrameProfile::empty())),
        }
    }

    pub fn cpu_zone(&self, name: &'static str) -> CpuZoneGuard<'_> {
        let mut inner = self.inner.borrow_mut();

        let id = inner.get_or_create_zone(name);
        let parent = inner.stack.last().map(|entry| entry.id);

        inner.stack.push(StackEntry { id, start: Instant::now() });

        CpuZoneGuard { profiler: self, id, parent }
    }

    pub fn cpu_meta(&self, name: &'static str, value: MetaValue) {
        let mut inner = self.inner.borrow_mut();

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
            let mut inner = self.inner.borrow_mut();

            debug_assert!(inner.stack.is_empty(), "frame ended with open zones");

            let profile = FrameProfile {
                zones: take(&mut inner.events),
                cpu_meta: take(&mut inner.cpu_meta),
                gpu_meta: Vec::new(),
            };

            inner.cpu_meta_index_by_name.clear();

            profile
        };

        *self.last_profile.borrow_mut() = Arc::new(profile);
    }

    pub fn last_profile(&self) -> Arc<FrameProfile> {
        self.last_profile.borrow().clone()
    }
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

pub struct CpuZoneGuard<'a> {
    profiler: &'a FrameProfiler,
    id: ZoneId,
    parent: Option<ZoneId>,
}

impl<'a> Drop for CpuZoneGuard<'a> {
    fn drop(&mut self) {
        let mut inner = self.profiler.inner.borrow_mut();

        let entry = inner.stack.pop().expect("zone stack underflow");
        debug_assert_eq!(entry.id, self.id);

        let duration_ns = entry.start.elapsed().as_nanos() as u64;
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
