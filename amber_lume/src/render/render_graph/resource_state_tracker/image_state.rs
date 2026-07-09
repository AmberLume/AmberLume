use ash::vk::{AccessFlags, ImageLayout, PipelineStageFlags};

#[derive(Copy, Clone)]
pub struct ImageState {
    pub layout: ImageLayout,
    pub access: AccessFlags,
    pub stage: PipelineStageFlags,
    pub write_access: AccessFlags,
    pub write_stage: PipelineStageFlags,
    pub write_layout: ImageLayout,
}

impl ImageState {
    pub fn undefined() -> Self {
        Self {
            layout: ImageLayout::UNDEFINED,
            access: AccessFlags::empty(),
            stage: PipelineStageFlags::TOP_OF_PIPE,
            write_access: AccessFlags::empty(),
            write_stage: PipelineStageFlags::empty(),
            write_layout: ImageLayout::UNDEFINED,
        }
    }
}
