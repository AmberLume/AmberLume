use ash::vk::{AccessFlags, PipelineStageFlags};
use gpu::BufferRange;

pub struct PendingBufferBarrier {
    pub buffer_range: BufferRange,
    pub src_access: AccessFlags,
    pub dst_access: AccessFlags,
    pub src_stage: PipelineStageFlags,
    pub dst_stage: PipelineStageFlags,
}
