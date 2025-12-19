use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::image::vulkan_image::VulkanImage;
use ash::vk::{
    AccessFlags, CommandBuffer, DependencyFlags, Extent2D, ImageLayout, ImageMemoryBarrier,
    PipelineStageFlags, QUEUE_FAMILY_IGNORED,
};
use glam::{Mat4, Vec3};

pub fn transition_image_layout(
    device_context: &DeviceContext,
    command_buffer: CommandBuffer,
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
        device_context.device.cmd_pipeline_barrier(
            command_buffer,
            src_stage,
            dst_stage,
            DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        )
    }
}

pub fn create_isometric_view_projection(
    height: f32,
    extent: &Extent2D,
    distance: f32,
    center: Vec3,
) -> Mat4 {
    let aspect_ratio = extent.width as f32 / extent.height as f32;
    let width = height * aspect_ratio;

    let half_width = width / 2.0;
    let half_height = height / 2.0;

    let projection = Mat4::orthographic_rh(
        -half_width,
        half_width,
        -half_height,
        half_height,
        0.1,
        1000.0,
    );

    let camera_pos = center + Vec3::new(0.0, distance * 0.707, distance * 0.707);

    let view = Mat4::look_at_rh(camera_pos, center, Vec3::Y);

    let vulkan_correction = Mat4::from_cols_array(&[
        1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
    ]);

    vulkan_correction * projection * view
}
