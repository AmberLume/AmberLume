use ash::vk::DeviceAddress;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DepthPushConstants {
    pub view_projection: [[f32; 4]; 4],

    pub vertex_buffer_address: DeviceAddress,
}
