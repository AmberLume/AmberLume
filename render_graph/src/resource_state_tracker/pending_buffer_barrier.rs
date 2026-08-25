use ash::vk::{AccessFlags, PipelineStageFlags};
use crate::resource_state_tracker::buffer_region_key::BufferRegionKey;

pub struct PendingBufferBarrier {
    pub region: BufferRegionKey,
    pub src_access: AccessFlags,
    pub dst_access: AccessFlags,
    pub src_stage: PipelineStageFlags,
    pub dst_stage: PipelineStageFlags,
}
