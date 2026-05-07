use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::render::buffer::typed::scene_buffer::SceneGPU;
use crate::render::factories::buffer::typed_buffer::typed_buffer::TypedBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::resources::store::providers::resource_provider::ResourceId;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct ShadowMaskPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    
    pub bias: f32,
    pub pcf_radius: i32,

    pub depth_descriptor_id: u32,
    pub global_shadow_descriptor_id: u32,

    _pad0: [u32; 26],
}

impl ShadowMaskPushConstants {
    pub fn create(
        scene_buffer: BufferView<TypedBuffer<SceneGPU>>,
        bias: f32,
        pcf_radius: i32,
        depth_descriptor_id: ResourceId,
        global_shadow_descriptor_id: ResourceId,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.get().device_address(),
            
            bias,
            pcf_radius,

            depth_descriptor_id,
            global_shadow_descriptor_id,
            
            _pad0: [0; 26],
        }
    }
}
