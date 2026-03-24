use ash::vk::{AttachmentLoadOp, AttachmentStoreOp, ClearColorValue, ClearDepthStencilValue, ClearValue, Extent2D, Extent3D, ImageLayout, ImageView, RenderingAttachmentInfo};

pub struct ImageAttachment<'a> {
    pub info: RenderingAttachmentInfo<'a>,
}

impl<'a> ImageAttachment<'a> {
    pub fn from(image_view: ImageView) -> Self {
        Self {
            info: RenderingAttachmentInfo::default()
                .image_view(image_view),
        }
    }

    pub fn layout(self, layout: ImageLayout) -> Self {
        Self {
            info: self.info
                .image_layout(layout),
        }
    }

    pub fn ops(self, load: AttachmentLoadOp, store: AttachmentStoreOp) -> Self {
        Self {
            info: self.info
                .load_op(load)
                .store_op(store),
        }
    }

    pub fn clear_color(self, color: [f32; 4]) -> Self {
        Self {
            info: self.info
                .clear_value(ClearValue { 
                    color: ClearColorValue { 
                        float32: color,
                    }, 
                }),
        }
    }

    pub fn clear_depth_stencil(self, depth: f32, stencil: u32) -> Self {
        Self {
            info: self.info
                .clear_value(ClearValue { 
                    depth_stencil: ClearDepthStencilValue { 
                        depth, 
                        stencil,
                    }, 
                }),
        }
    }
}

pub fn extent_3d_to_2d(extent: Extent3D) -> Extent2D {
    Extent2D {
        width: extent.width,
        height: extent.height,
    }
} 
