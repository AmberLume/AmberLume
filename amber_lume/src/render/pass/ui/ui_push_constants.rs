use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct UiPushConstants {
    pub ui_vertex_buffer_device_address: DeviceAddress,
    
    pub texture_index: u32,
    pub render_mode: u32,
}

impl UiPushConstants {
    pub fn create(
        ui_vertex_buffer_device_address: DeviceAddress,
        texture_index: u32,
        render_mode: u32,
    ) -> Self {
        Self {
            ui_vertex_buffer_device_address,
            
            texture_index,
            render_mode,
        }
    }
}
