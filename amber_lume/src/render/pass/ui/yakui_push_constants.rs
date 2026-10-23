use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct YakuiPushConstants {
    pub ui_vertex_buffer_device_address: DeviceAddress,

    pub texture_index: u32,
    pub render_mode: u32,

    _pad0: [u32; 28],
}

impl YakuiPushConstants {
    pub fn create(
        ui_vertex_buffer: BufferRange,
        texture_index: u32,
        render_mode: u32,
    ) -> Self {
        Self {
            ui_vertex_buffer_device_address: ui_vertex_buffer.device_address,

            texture_index,
            render_mode,

            _pad0: [0; 28],
        }
    }
}
