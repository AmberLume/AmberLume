use ash::vk::{AccessFlags, PipelineStageFlags};

#[derive(Copy, Clone)]
pub struct BufferState {
    pub access: AccessFlags,
    pub stage: PipelineStageFlags,
}

impl BufferState {
    pub const WRITE_ACCESS: AccessFlags = AccessFlags::from_raw(
        AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR.as_raw()
            | AccessFlags::SHADER_WRITE.as_raw()
            | AccessFlags::TRANSFER_WRITE.as_raw()
            | AccessFlags::HOST_WRITE.as_raw()
            | AccessFlags::MEMORY_WRITE.as_raw(),
    );

    pub fn initial() -> Self {
        Self {
            access: AccessFlags::empty(),
            stage: PipelineStageFlags::TOP_OF_PIPE,
        }
    }
}
