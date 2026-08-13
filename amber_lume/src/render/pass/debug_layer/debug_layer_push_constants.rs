use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DebugLayerPushConstants {
    pub scene_buffer_device_address: DeviceAddress,

    pub texture_index: u32,
    pub layer_kind: u32,
    pub shadow_colored: u32,

    _pad0: [u32; 27],
}

impl DebugLayerPushConstants {
    pub fn create(
        scene_buffer_device_address: DeviceAddress,
        texture_index: u32,
        layer_kind: u32,
        shadow_colored: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address,

            texture_index,
            layer_kind,
            shadow_colored,

            _pad0: [0; 27],
        }
    }
}
