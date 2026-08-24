use crate::store::resource_store::ResourceStore;
use gpu::BufferRange;

pub struct ResourceBuffers {
    pub index_buffer: BufferRange,
    pub vertex_buffer: BufferRange,
    pub submesh_buffer: BufferRange,
    pub mesh_buffer: BufferRange,

    pub skeleton_buffer: BufferRange,
    pub skeleton_bone_buffer: BufferRange,

    pub animation_buffer: BufferRange,
    pub animation_frame_buffer: BufferRange,

    pub material_buffer: BufferRange,
}

impl ResourceBuffers {
    pub fn from_store(resource_store: &ResourceStore) -> Self {
        let mesh_backend = &resource_store.mesh_provider.backend;
        let skeleton_backend = &resource_store.skeletons_provider.backend;
        let animation_backend = &resource_store.animation_provider.backend;
        let material_backend = &resource_store.material_provider.backend;

        Self {
            index_buffer: mesh_backend.index_buffer.whole(),
            vertex_buffer: mesh_backend.vertex_buffer.whole(),
            submesh_buffer: mesh_backend.submesh_buffer.whole(),
            mesh_buffer: mesh_backend.mesh_buffer.whole(),

            skeleton_buffer: skeleton_backend.skeletons_buffer.whole(),
            skeleton_bone_buffer: skeleton_backend.skeleton_bones_buffer.whole(),

            animation_buffer: animation_backend.animation_buffer.whole(),
            animation_frame_buffer: animation_backend.animation_frame_buffer.whole(),

            material_buffer: material_backend.material_buffer.whole(),
        }
    }
}
