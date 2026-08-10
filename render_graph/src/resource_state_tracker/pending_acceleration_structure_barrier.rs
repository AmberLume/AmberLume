use ash::vk::{AccessFlags, PipelineStageFlags};

pub struct PendingAccelerationStructureBarrier {
    pub src_access: AccessFlags,
    pub dst_access: AccessFlags,
    pub src_stage: PipelineStageFlags,
    pub dst_stage: PipelineStageFlags,
}
