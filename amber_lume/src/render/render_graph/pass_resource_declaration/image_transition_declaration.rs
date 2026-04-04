use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use ash::vk::{AccessFlags, ImageLayout, PipelineStageFlags};

pub struct ImageTransitionDeclaration {
    pub image: VirtualImage,
    pub layout: ImageLayout,
    pub access: AccessFlags,
    pub stage: PipelineStageFlags,
}

impl ImageTransitionDeclaration {
    pub fn new(
        image: VirtualImage,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> Self {
        Self {
            image,
            layout,
            access,
            stage,
        }
    }
}
