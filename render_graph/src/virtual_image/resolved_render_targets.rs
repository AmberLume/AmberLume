use crate::frame_context::FrameContext;
use ash::vk::{
    Extent2D, Offset2D,
    Rect2D, RenderingAttachmentInfo, RenderingInfo,
};
use crate::virtual_image::resolved_attachment::ResolvedAttachment;

pub struct ResolvedRenderTargets {
    color: Vec<ResolvedAttachment>,
    depth: Option<ResolvedAttachment>,
    extent: Extent2D,
    view_mask: u32,
}

impl ResolvedRenderTargets {
    pub fn new(
        color: Vec<ResolvedAttachment>,
        depth: Option<ResolvedAttachment>,
        extent: Extent2D,
        view_mask: u32,
    ) -> Self {
        Self {
            color,
            depth,
            extent,
            view_mask,
        }
    }

    pub fn open(&self, frame_context: &FrameContext) {
        let color_attachments: Vec<RenderingAttachmentInfo> = self.color.iter()
            .map(|attachment| attachment.to_info())
            .collect();

        let depth_attachment = self.depth.as_ref()
            .map(|attachment| attachment.to_info());

        let mut rendering_info = RenderingInfo::default()
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: self.extent,
            })
            .layer_count(1)
            .view_mask(self.view_mask)
            .color_attachments(&color_attachments);

        if let Some(depth_attachment) = &depth_attachment {
            rendering_info = rendering_info.depth_attachment(depth_attachment);
        }

        frame_context.begin_rendering(&rendering_info);
        frame_context.set_render_area(self.extent);
    }
}
