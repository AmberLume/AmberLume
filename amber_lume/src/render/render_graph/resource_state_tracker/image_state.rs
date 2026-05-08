use ash::vk::{AccessFlags, ImageLayout, PipelineStageFlags};

#[derive(Copy, Clone)]
pub struct ImageState {
    pub layout: ImageLayout,
    pub access: AccessFlags,
    pub stage: PipelineStageFlags,
}

impl ImageState {
    pub fn undefined() -> Self {
        Self {
            layout: ImageLayout::UNDEFINED,
            access: AccessFlags::empty(),
            stage: PipelineStageFlags::TOP_OF_PIPE,
        }
    }
}
