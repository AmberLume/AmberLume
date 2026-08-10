use ash::vk::{AccessFlags, Buffer, DeviceSize, PipelineStageFlags};

pub struct PendingBufferBarrier {
    pub buffer: Buffer,
    pub offset: DeviceSize,
    pub size: DeviceSize,
    pub src_access: AccessFlags,
    pub dst_access: AccessFlags,
    pub src_stage: PipelineStageFlags,
    pub dst_stage: PipelineStageFlags,
}
