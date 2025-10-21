use crate::render::vulkan::image::depth_gpu_image::DepthImage;
use crate::render::vulkan::image::gpu_image::GpuImage;
use crate::render::vulkan::swapchain::Swapchain;
use anyhow::Result;
use ash::vk::{Extent2D, ImageView, PhysicalDevice};
use ash::{Device, Instance, vk};
use std::slice;
use vk::{
    AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp,
    Format, Framebuffer, FramebufferCreateInfo, ImageLayout, PipelineBindPoint, PipelineStageFlags,
    RenderPass, RenderPassCreateInfo, SUBPASS_EXTERNAL, SampleCountFlags, SubpassDependency,
    SubpassDescription,
};

pub struct RenderTargets {
    pub render_pass: RenderPass,
    pub framebuffers: Vec<Framebuffer>,
    pub depth_gpu_images: Vec<GpuImage>,
}

impl RenderTargets {
    pub fn create(
        instance: &Instance,
        device: &Device,
        physical_device: PhysicalDevice,
        swapchain: &Swapchain,
    ) -> Result<Self> {
        let msaa = SampleCountFlags::TYPE_1;
        let count = swapchain.image_views.len();

        let (depth_format, depth_gpu_images) = create_depth_images(
            &instance,
            &device,
            physical_device,
            swapchain.extent,
            count,
            SampleCountFlags::TYPE_1,
        )?;
        let depth_image_views: Vec<ImageView> = depth_gpu_images
            .iter()
            .map(|image_view| image_view.image_view)
            .collect();

        let render_pass = create_render_pass(&device, swapchain.format, depth_format, msaa)?;

        let framebuffers = create_framebuffers(
            &device,
            render_pass,
            swapchain.extent,
            &swapchain.image_views,
            &depth_image_views,
        )?;

        let render_targets = Self {
            render_pass,
            framebuffers,
            depth_gpu_images,
        };

        Ok(render_targets)
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            for &framebuffer in &self.framebuffers {
                device.destroy_framebuffer(framebuffer, None);
            }
            for depth_image_view in &self.depth_gpu_images {
                depth_image_view.destroy(device);
            }

            device.destroy_render_pass(self.render_pass, None);
        }
    }
}

fn create_render_pass(
    device: &Device,
    color_format: Format,
    depth_format: Format,
    sample_count_flags: SampleCountFlags,
) -> Result<RenderPass> {
    let color_attachment_description = AttachmentDescription::default()
        .format(color_format)
        .samples(sample_count_flags)
        .load_op(AttachmentLoadOp::CLEAR)
        .store_op(AttachmentStoreOp::STORE)
        .initial_layout(ImageLayout::UNDEFINED)
        .final_layout(ImageLayout::PRESENT_SRC_KHR);

    let color_attachment_reference = AttachmentReference::default()
        .attachment(0)
        .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let depth_attachment_description = AttachmentDescription::default()
        .format(depth_format)
        .samples(sample_count_flags)
        .load_op(AttachmentLoadOp::CLEAR)
        .store_op(AttachmentStoreOp::STORE)
        .stencil_load_op(AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(AttachmentStoreOp::DONT_CARE)
        .initial_layout(ImageLayout::UNDEFINED)
        .final_layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let depth_attachment_reference = AttachmentReference::default()
        .attachment(1)
        .layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let subpass_description = SubpassDescription::default()
        .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
        .color_attachments(slice::from_ref(&color_attachment_reference))
        .depth_stencil_attachment(&depth_attachment_reference);

    let subpass_dependency = SubpassDependency::default()
        .src_subpass(SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .dst_stage_mask(
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .dst_access_mask(
            AccessFlags::COLOR_ATTACHMENT_WRITE | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        );

    let attachment_descriptions = [color_attachment_description, depth_attachment_description];
    let subpass_dependencies = [subpass_dependency];
    let render_pass_create_info = RenderPassCreateInfo::default()
        .attachments(&attachment_descriptions)
        .subpasses(slice::from_ref(&subpass_description))
        .dependencies(&subpass_dependencies);

    let render_pass = unsafe { device.create_render_pass(&render_pass_create_info, None)? };

    Ok(render_pass)
}

fn create_framebuffers(
    device: &Device,
    render_pass: RenderPass,
    extent: Extent2D,
    color_image_views: &[ImageView],
    depth_images_views: &[ImageView],
) -> Result<Vec<Framebuffer>> {
    let size = color_image_views.len();

    let mut vec = Vec::with_capacity(size);

    for index in 0..size {
        let image_view = color_image_views[index];
        let depth_image_view = depth_images_views[index];

        let attachments = [image_view, depth_image_view];

        let frame_buffer_create_info = FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);

        let framebuffer = unsafe { device.create_framebuffer(&frame_buffer_create_info, None) }?;

        vec.push(framebuffer)
    }

    Ok(vec)
}

fn create_depth_images(
    instance: &Instance,
    device: &Device,
    physical_device: PhysicalDevice,
    extent: Extent2D,
    count: usize,
    samples: SampleCountFlags,
) -> Result<(Format, Vec<GpuImage>)> {
    let mut vec = Vec::with_capacity(count);

    let format = DepthImage::find_depth_format(&instance, physical_device)?;

    for _ in 0..count {
        let image =
            DepthImage::create(&instance, &device, physical_device, format, extent, samples)?;

        vec.push(image)
    }

    Ok((format, vec))
}
