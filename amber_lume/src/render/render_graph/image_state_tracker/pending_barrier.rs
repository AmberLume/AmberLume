use ash::vk::{AccessFlags, Image, ImageLayout, ImageSubresourceRange, PipelineStageFlags};

pub struct PendingBarrier {
    pub image: Image,
    pub subresource_range: ImageSubresourceRange,
    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,
    pub src_access: AccessFlags,
    pub dst_access: AccessFlags,
    pub src_stage: PipelineStageFlags,
    pub dst_stage: PipelineStageFlags,
}
