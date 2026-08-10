use ash::vk::{Buffer, DeviceAddress};
use gpu::BufferInfo;
use index_allocator::SliceIndex;
use crate::store::resource_store::ResourceStore;

pub struct ResourceBuffers {
    pub index_buffer_handle: Buffer,

    pub mesh_buffer: DeviceAddress,
    pub submesh_buffer: DeviceAddress,
    pub index_buffer: DeviceAddress,
    pub vertex_buffer: DeviceAddress,

    pub skeleton_buffer: DeviceAddress,
    pub skeleton_bone_buffer: DeviceAddress,

    pub animation_buffer: DeviceAddress,
    pub animation_frame_buffer: DeviceAddress,

    pub material_buffer: DeviceAddress,
}

impl ResourceBuffers {
    pub fn from_store(resource_store: &ResourceStore) -> Self {
        let mesh_backend = &resource_store.mesh_provider.backend;
        let skeleton_backend = &resource_store.skeletons_provider.backend;
        let animation_backend = &resource_store.animation_provider.backend;
        let material_backend = &resource_store.material_provider.backend;

        Self {
            index_buffer_handle: mesh_backend.index_buffer.handle(),

            mesh_buffer: mesh_backend.mesh_buffer.slice_at(SliceIndex::ZERO).device_address(),
            submesh_buffer: mesh_backend.submesh_buffer.slice_at(SliceIndex::ZERO).device_address(),
            index_buffer: mesh_backend.index_buffer.slice_at(SliceIndex::ZERO).device_address(),
            vertex_buffer: mesh_backend.vertex_buffer.slice_at(SliceIndex::ZERO).device_address(),

            skeleton_buffer: skeleton_backend.skeletons_buffer.slice_at(SliceIndex::ZERO).device_address(),
            skeleton_bone_buffer: skeleton_backend.skeleton_bones_buffer.slice_at(SliceIndex::ZERO).device_address(),

            animation_buffer: animation_backend.animation_buffer.slice_at(SliceIndex::ZERO).device_address(),
            animation_frame_buffer: animation_backend.animation_frame_buffer.slice_at(SliceIndex::ZERO).device_address(),

            material_buffer: material_backend.material_buffer.slice_at(SliceIndex::ZERO).device_address(),
        }
    }
}
