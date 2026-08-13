use ash::vk::{AccessFlags, PipelineStageFlags};

#[derive(Copy, Clone)]
pub struct BufferState {
    pub access: AccessFlags,
    pub stage: PipelineStageFlags,
}

impl BufferState {
    pub fn initial() -> Self {
        Self {
            access: AccessFlags::empty(),
            stage: PipelineStageFlags::TOP_OF_PIPE,
        }
    }
}
