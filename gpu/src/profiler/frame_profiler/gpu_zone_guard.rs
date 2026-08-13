use ash::vk::CommandBuffer;
use index_allocator::FrameIndex;
use crate::profiler::frame_profiler::frame_profiler::FrameProfiler;
use crate::profiler::gpu_profiler::pending_gpu_zone::PendingGpuZone;
use crate::profiler::zone::zone_id::ZoneId;

pub struct GpuZoneGuard<'a> {
    pub(super) profiler: &'a FrameProfiler,
    pub(super) cmd: CommandBuffer,
    pub(super) zone_id: ZoneId,
    pub(super) parent: Option<ZoneId>,
    pub(super) frame_index: FrameIndex,
    pub(super) slot: u32,
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
