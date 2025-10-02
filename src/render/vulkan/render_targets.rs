use super::logical_device::LogicalDevice;
use anyhow::Result;
use ash::{Device, vk};
use std::slice;
use vk::{
    AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp,
    Extent2D, Format, Framebuffer, FramebufferCreateInfo, ImageLayout, ImageView,
    PipelineBindPoint, PipelineStageFlags, RenderPass, RenderPassCreateInfo, SUBPASS_EXTERNAL,
    SampleCountFlags, SubpassDependency, SubpassDescription,
};

pub struct RenderTargets {
    pub render_pass: RenderPass,
    pub framebuffers: Vec<Framebuffer>,
}

impl RenderTargets {
    pub fn create(
        logical_device: &LogicalDevice,
        format: Format,
        views: &[ImageView],
        extent: Extent2D,
    ) -> Result<Self> {
        let render_pass = create_render_pass(&logical_device.device, format)?;
        let framebuffers = create_framebuffers(&logical_device.device, render_pass, views, extent)?;

        let render_targets = Self {
            render_pass,
            framebuffers,
        };

        Ok(render_targets)
    }

    pub fn destroy(&self, logical_device: &LogicalDevice) {
        unsafe {
            for &framebuffer in &self.framebuffers {
                logical_device.device.destroy_framebuffer(framebuffer, None);
            }

            logical_device
                .device
                .destroy_render_pass(self.render_pass, None);
        }
    }
}

fn create_render_pass(dev: &Device, format: Format) -> Result<RenderPass> {
    let attachment_description = AttachmentDescription::default()
        .format(format)
        .samples(SampleCountFlags::TYPE_1)
        .load_op(AttachmentLoadOp::CLEAR)
        .store_op(AttachmentStoreOp::STORE)
        .initial_layout(ImageLayout::UNDEFINED)
        .final_layout(ImageLayout::PRESENT_SRC_KHR);

    let attachment_reference = AttachmentReference::default()
        .attachment(0)
        .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let subpass_description = SubpassDescription::default()
        .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
        .color_attachments(slice::from_ref(&attachment_reference));

    let subpass_dependency = [SubpassDependency::default()
        .src_subpass(SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(AccessFlags::COLOR_ATTACHMENT_WRITE)];

    let render_pass_create_info = RenderPassCreateInfo::default()
        .attachments(slice::from_ref(&attachment_description))
        .subpasses(slice::from_ref(&subpass_description))
        .dependencies(&subpass_dependency);

    let render_pass = unsafe { dev.create_render_pass(&render_pass_create_info, None)? };

    Ok(render_pass)
}

fn create_framebuffers(
    device: &Device,
    render_pass: RenderPass,
    views: &[ImageView],
    extent: Extent2D,
) -> Result<Vec<Framebuffer>> {
    views
        .iter()
        .map(|&image_view| {
            let frame_buffer_create_info = FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(slice::from_ref(&image_view))
                .width(extent.width)
                .height(extent.height)
                .layers(1);

            unsafe { device.create_framebuffer(&frame_buffer_create_info, None) }
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}
