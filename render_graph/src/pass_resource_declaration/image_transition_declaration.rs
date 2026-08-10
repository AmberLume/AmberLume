use crate::virtual_image::virtual_image::VirtualImage;
use ash::vk::{AccessFlags, ImageLayout, PipelineStageFlags};

pub struct ImageTransitionDeclaration {
    pub image: VirtualImage,
    pub mip: Option<u32>,
    pub layout: ImageLayout,
    pub access: AccessFlags,
    pub stage: PipelineStageFlags,
}

impl ImageTransitionDeclaration {
    pub fn new(
        image: VirtualImage,
        mip: Option<u32>,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> Self {
        Self {
            image,
            mip,
            layout,
            access,
            stage,
        }
    }
}
