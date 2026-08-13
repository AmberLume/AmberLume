use crate::profiler::frame_profile::zone_entry::ZoneEntry;
use crate::profiler::frame_profiler::frame_profiler::FrameProfiler;
use crate::profiler::zone::zone_id::ZoneId;
use crate::profiler::zone::zone_kind::ZoneKind;

pub struct CpuZoneGuard<'a> {
    pub(super) profiler: &'a FrameProfiler,
    pub(super) id: ZoneId,
    pub(super) parent: Option<ZoneId>,
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
