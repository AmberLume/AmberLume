use std::collections::HashMap;
use std::mem::take;
use anyhow::{bail, Result};
use ash::vk::{BufferUsageFlags, CommandBuffer, PipelineStageFlags};
use gpu_allocator::MemoryLocation;
use crate::ids::{FrameIndex, SliceIndex};
use crate::profiler::zone::ZoneId;
use crate::render::device::device_context::DeviceContext;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::query_pool::query_pool::ManagedQueryPool;
use crate::render::factories::resource_factories::ResourceFactories;

const QUERIES_PER_ZONE: u32 = 2;

pub struct GpuProfiler {
    query_pool: ManagedQueryPool,
    readback_buffer: FrameBuffer<SliceBuffer<u64>>,

    max_zones: u32,
    timestamp_period: f64,

    slot_by_zone: HashMap<ZoneId, u32>,
    pending_per_frame: Vec<Vec<PendingGpuZone>>,
}

pub struct PendingGpuZone {
    pub zone_id: ZoneId,
    pub parent: Option<ZoneId>,
    pub slot: u32,
}

pub struct ResolvedGpuZone {
    pub zone_id: ZoneId,
    pub parent: Option<ZoneId>,
    pub duration_ns: u64,
}

impl GpuProfiler {
    pub fn new(
        device_context: &DeviceContext,
        resource_factories: &ResourceFactories,
        frames_in_flight: u32,
        max_zones: u32,
    ) -> Result<Self> {
        let total_queries = frames_in_flight * max_zones * QUERIES_PER_ZONE;

        let query_pool = resource_factories
            .query_pool_factory
            .create_query_pool(total_queries, "frame_profiler")?;

        let readback_buffer = BufferBuilder::slice::<u64>(max_zones * QUERIES_PER_ZONE)
            .per_frame(frames_in_flight)
            .build(
                &resource_factories.buffer_factory,
                "frame_profiler",
                BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST,
                MemoryLocation::GpuToCpu,
            )?;

        Ok(Self {
            query_pool,
            readback_buffer,

            max_zones,
            timestamp_period: device_context.physical_device_info.timestamp_period as f64,

            slot_by_zone: HashMap::new(),
            pending_per_frame: (0..frames_in_flight).map(|_| Vec::new()).collect(),
        })
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        self.query_pool.destroy();
        resource_factories
            .buffer_factory
            .destroy_buffer(self.readback_buffer.into_managed_buffer())?;

        Ok(())
    }

    pub fn allocate_slot(&mut self, zone_id: ZoneId) -> Result<u32> {
        if let Some(&slot) = self.slot_by_zone.get(&zone_id) {
            return Ok(slot);
        }

        let slot = self.slot_by_zone.len() as u32;
        if slot >= self.max_zones {
            bail!("FrameProfiler GPU zone capacity exceeded ({})", self.max_zones);
        }

        self.slot_by_zone.insert(zone_id, slot);

        Ok(slot)
    }

    pub fn write_start(&self, cmd: CommandBuffer, frame_index: FrameIndex, slot: u32) {
        let base = self.query_base(frame_index, slot);

        self.query_pool.reset(cmd, base, QUERIES_PER_ZONE);
        self.query_pool.record(cmd, PipelineStageFlags::TOP_OF_PIPE, base);
    }

    pub fn write_end(&self, cmd: CommandBuffer, frame_index: FrameIndex, slot: u32) {
        let base = self.query_base(frame_index, slot);

        self.query_pool.record(cmd, PipelineStageFlags::BOTTOM_OF_PIPE, base + 1);
    }

    pub fn record_pending(&mut self, frame_index: FrameIndex, zone: PendingGpuZone) {
        self.pending_per_frame[frame_index.value as usize].push(zone);
    }

    pub fn extract(&self, cmd: CommandBuffer, frame_index: FrameIndex) {
        for zone in &self.pending_per_frame[frame_index.value as usize] {
            let buffer_view = self
                .readback_buffer
                .frame(frame_index)
                .slice_at(SliceIndex::from(zone.slot * QUERIES_PER_ZONE));

            self.query_pool.copy_to_buffer::<u64>(
                cmd,
                self.query_base(frame_index, zone.slot),
                QUERIES_PER_ZONE,
                &buffer_view,
            );
        }
    }

    pub fn collect_previous(&mut self, frame_index: FrameIndex) -> Vec<ResolvedGpuZone> {
        let pending = take(&mut self.pending_per_frame[frame_index.value as usize]);
        if pending.is_empty() {
            return Vec::new();
        }

        pending
            .into_iter()
            .map(|zone| {
                let buffer_view = self
                    .readback_buffer
                    .frame(frame_index)
                    .slice_at(SliceIndex::from(zone.slot * QUERIES_PER_ZONE));
                let mapped_ptr = buffer_view.mapped_ptr() as *const u64;

                let (start, end) = unsafe { (mapped_ptr.read(), mapped_ptr.add(1).read()) };
                let duration_ns = (end.saturating_sub(start) as f64 * self.timestamp_period) as u64;

                ResolvedGpuZone {
                    zone_id: zone.zone_id,
                    parent: zone.parent,
                    duration_ns,
                }
            })
            .collect()
    }

    fn query_base(&self, frame_index: FrameIndex, slot: u32) -> u32 {
        frame_index.value * self.max_zones * QUERIES_PER_ZONE + slot * QUERIES_PER_ZONE
    }
}
