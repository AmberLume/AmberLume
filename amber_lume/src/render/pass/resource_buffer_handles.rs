use render_graph::PassGraph;
use render_graph::VirtualBuffer;
use resource_store::ResourceBuffers;

#[derive(Clone, Copy)]
pub struct ResourceBufferHandles {
    pub index_buffer: VirtualBuffer,
    pub vertex_buffer: VirtualBuffer,
    pub submesh_buffer: VirtualBuffer,
    pub mesh_buffer: VirtualBuffer,

    pub skeleton_buffer: VirtualBuffer,
    pub skeleton_bone_buffer: VirtualBuffer,

    pub animation_buffer: VirtualBuffer,
    pub animation_frame_buffer: VirtualBuffer,

    pub material_buffer: VirtualBuffer,
}

impl ResourceBufferHandles {
    pub fn import(pass_graph: &mut PassGraph, resource_buffers: &ResourceBuffers) -> Self {
        Self {
            index_buffer: pass_graph.import_buffer(resource_buffers.index_buffer),
            vertex_buffer: pass_graph.import_buffer(resource_buffers.vertex_buffer),
            submesh_buffer: pass_graph.import_buffer(resource_buffers.submesh_buffer),
            mesh_buffer: pass_graph.import_buffer(resource_buffers.mesh_buffer),

            skeleton_buffer: pass_graph.import_buffer(resource_buffers.skeleton_buffer),
            skeleton_bone_buffer: pass_graph.import_buffer(resource_buffers.skeleton_bone_buffer),

            animation_buffer: pass_graph.import_buffer(resource_buffers.animation_buffer),
            animation_frame_buffer: pass_graph.import_buffer(resource_buffers.animation_frame_buffer),

            material_buffer: pass_graph.import_buffer(resource_buffers.material_buffer),
        }
    }
}
