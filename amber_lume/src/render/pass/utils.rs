use ash::vk::{AttachmentLoadOp, AttachmentStoreOp, ClearColorValue, ClearDepthStencilValue, ClearValue, ImageLayout, ImageView, RenderingAttachmentInfo};
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;

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

impl VirtualBuffer {
    pub fn stage_slice<T>(
        self,
        resource_registry: &mut ResourceRegistry,
        allocator: &mut HeapAllocator,
        data: &[T],
    ) -> anyhow::Result<()> {
        let physical_buffer = allocator.allocate_for_slice(&data)?;
        physical_buffer.write(&data)?;

        resource_registry.update_imported_buffer(
            self,
            physical_buffer.buffer,
            physical_buffer.offset,
            physical_buffer.size,
            physical_buffer.device_address,
            physical_buffer.mapped_ptr,
        );

        Ok(())
    }
}
