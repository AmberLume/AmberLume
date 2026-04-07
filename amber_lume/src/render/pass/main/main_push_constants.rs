use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::ids::SliceIndex;
use crate::render::buffer::typed::draw_data_buffer::DrawDataGPU;
use crate::render::buffer::typed::entity_buffer::EntityGPU;
use crate::render::buffer::typed::scene_buffer::SceneGPU;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::buffer::typed_buffer::typed_buffer::TypedBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::resources::store::providers::resource_provider::ResourceId;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct MainPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub submesh_buffer_device_address: DeviceAddress,
    pub material_buffer_device_address: DeviceAddress,
    pub bone_transform_buffer_device_address: DeviceAddress,
    
    pub shadow_mask_resource_id: ResourceId,

    _pad0: u32,
}

impl MainPushConstants {
    pub fn create(
        scene_buffer: BufferView<TypedBuffer<SceneGPU>>,
        draw_data_buffer: BufferView<SliceBuffer<DrawDataGPU>>,
        vertex_buffer_device_address: DeviceAddress,
        entity_buffer: BufferView<SliceBuffer<EntityGPU>>,
        submesh_buffer_device_address: DeviceAddress,
        material_buffer_device_address: DeviceAddress,
        bone_transform_buffer_device_address: DeviceAddress,
        shadow_mask_resource_id: ResourceId,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.get().device_address(),
            draw_data_buffer_device_address: draw_data_buffer.slice_at(SliceIndex::ZERO).device_address(),
            vertex_buffer_device_address,
            entity_buffer_device_address: entity_buffer.slice_at(SliceIndex::ZERO).device_address(),
            submesh_buffer_device_address,
            material_buffer_device_address,
            bone_transform_buffer_device_address,

            shadow_mask_resource_id,

            _pad0: 0,
        }
    }
}
