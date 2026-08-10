use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use render_graph::PhysicalBuffer;

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct PhysicsDebugPushConstants {
    pub scene_buffer_device_address: DeviceAddress,

    pub physics_debug_vertex_buffer_device_address: DeviceAddress,

    _pad0: [u32; 28],
}

impl PhysicsDebugPushConstants {
    pub fn create(
        scene_buffer: PhysicalBuffer,
        physics_debug_vertex_buffer: PhysicalBuffer,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,

            physics_debug_vertex_buffer_device_address: physics_debug_vertex_buffer.device_address,

            _pad0: [0; 28],
        }
    }
}
