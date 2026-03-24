use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use yakui::paint::Vertex;
use crate::ids::SliceIndex;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct UiPushConstants {
    pub ui_vertex_buffer_device_address: DeviceAddress,
    
    pub texture_index: u32,
    pub render_mode: u32,
}

impl UiPushConstants {
    pub fn create(
        ui_vertex_buffer: BufferView<SliceBuffer<Vertex>>,
        texture_index: u32,
        render_mode: u32,
    ) -> Self {
        Self {
            ui_vertex_buffer_device_address: ui_vertex_buffer.slice_at(SliceIndex::ZERO).device_address(),
            
            texture_index,
            render_mode,
        }
    }
}
