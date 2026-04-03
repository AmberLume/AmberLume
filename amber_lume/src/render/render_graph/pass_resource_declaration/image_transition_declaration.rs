use ash::vk::{AccessFlags, Image, ImageLayout, ImageSubresourceRange, PipelineStageFlags};

pub struct ImageTransitionDeclaration {
    pub image: Image,
    pub subresource_range: ImageSubresourceRange,
    pub layout: ImageLayout,
    pub access: AccessFlags,
    pub stage: PipelineStageFlags,
}
