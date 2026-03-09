use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use yakui::paint::Vertex;
use crate::render::vulkan::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::vulkan::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::vulkan::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct UiVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

pub fn create_ui_vertex_buffer(
    buffer_factory: &ManagedBufferFactory,
    frame_count: u32,
    capacity: u32,
) -> Result<FrameBuffer<SliceBuffer<Vertex>>> {
    BufferBuilder::slice(capacity)
        .per_frame(frame_count)
        .build(
            buffer_factory,
            "ui_vertex_buffer",
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::CpuToGpu,
        )
}
