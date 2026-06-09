use std::mem::size_of;
use std::sync::atomic::{AtomicU32, Ordering};
use ash::vk::{AccessFlags, ImageLayout, PipelineStageFlags};
use crate::render::pass::pass_context::PassContext;
use crate::render::readback::gpu_readback::GpuReadback;
use crate::render::readback::readbacks::ReadbackSlice;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;

const NO_ENTITY: u32 = u32::MAX;

pub struct EntityIdPickReader {
    source: VirtualImage,

    value: AtomicU32,
}

impl EntityIdPickReader {
    pub fn create(source: VirtualImage) -> Self {
        Self {
            source,

            value: AtomicU32::new(NO_ENTITY),
        }
    }

    pub fn value(&self) -> Option<u32> {
        match self.value.load(Ordering::Relaxed) {
            NO_ENTITY => None,
            index => Some(index),
        }
    }
}

impl GpuReadback for EntityIdPickReader {
    fn size(&self) -> u32 {
        size_of::<u32>() as u32
    }

    fn declare(&self, declaration: &mut PassResourceDeclaration) {
        declaration.read_image(
            self.source,
            ImageLayout::TRANSFER_SRC_OPTIMAL,
            AccessFlags::TRANSFER_READ,
            PipelineStageFlags::TRANSFER,
        );
    }

    fn record(&self, context: &PassContext, resource_registry: &ResourceRegistry, slice: &ReadbackSlice) {
        let image = resource_registry.get_physical_image(self.source);

        let width = image.extent.width.max(1);
        let height = image.extent.height.max(1);

        let x = (width as i32 / 2).clamp(0, width as i32 - 1);
        let y = (height as i32 / 2).clamp(0, height as i32 - 1);

        let (buffer, offset) = slice.copy_target();
        context.copy_image_texel_to_buffer(image.image, x, y, buffer, offset);
    }

    fn sync(&self, slice: &ReadbackSlice) {
        let value = unsafe { *(slice.mapped_ptr() as *const u32) };

        self.value.store(value, Ordering::Relaxed);
    }
}
