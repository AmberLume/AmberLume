use crate::virtual_buffer::virtual_buffer::VirtualBuffer;
use ash::vk::{AccessFlags, PipelineStageFlags};

pub struct BufferTransitionDeclaration {
    pub buffer: VirtualBuffer,
    pub access: AccessFlags,
    pub stage: PipelineStageFlags,
}

impl BufferTransitionDeclaration {
    pub fn new(buffer: VirtualBuffer, access: AccessFlags, stage: PipelineStageFlags) -> Self {
        Self {
            buffer,
            access,
            stage,
        }
    }
}
