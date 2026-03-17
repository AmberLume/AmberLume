use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct PhysicsDebugPushConstants {
    pub view_projection: [[f32; 4]; 4],

    pub physics_debug_vertex_buffer_device_address: DeviceAddress,
}

impl PhysicsDebugPushConstants {
    pub fn create(
        view_projection: [[f32; 4]; 4],
        physics_debug_vertex_buffer_device_address: DeviceAddress,
    ) -> Self {
        Self {
            view_projection,

            physics_debug_vertex_buffer_device_address,
        }
    }
}
