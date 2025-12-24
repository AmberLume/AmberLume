use crate::render::vulkan::image::vulkan_image::VulkanImage;
use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use ash::vk::{
    AccessFlags, DependencyFlags, ImageLayout, ImageMemoryBarrier, PipelineStageFlags,
    QUEUE_FAMILY_IGNORED,
};

pub fn transition_image_layout(
    render_pass_context: &RenderPassContext,
    vulkan_image: &VulkanImage,
    old_layout: ImageLayout,
    new_layout: ImageLayout,
    src_access: AccessFlags,
    dst_access: AccessFlags,
    src_stage: PipelineStageFlags,
    dst_stage: PipelineStageFlags,
) {
    let barrier = ImageMemoryBarrier::default()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(QUEUE_FAMILY_IGNORED)
        .image(vulkan_image.image)
        .subresource_range(vulkan_image.image_subresource_range)
        .src_access_mask(src_access)
        .dst_access_mask(dst_access);

    unsafe {
        render_pass_context
            .device_context
            .device
            .cmd_pipeline_barrier(
                render_pass_context.command_recording.command_buffer,
                src_stage,
                dst_stage,
                DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
    }
}
