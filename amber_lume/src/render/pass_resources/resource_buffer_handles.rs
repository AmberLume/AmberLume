use render_graph::PassGraph;
use render_graph::VirtualBuffer;
use resource_store::ResourceBuffers;

#[derive(Clone, Copy)]
pub struct ResourceBufferHandles {
    pub index_buffer: VirtualBuffer,
    pub mesh_vertex_buffer: VirtualBuffer,
    pub mesh_vertex_attribute_buffer: VirtualBuffer,
    pub mesh_vertex_skin_buffer: VirtualBuffer,
    pub submesh_buffer: VirtualBuffer,
    pub mesh_buffer: VirtualBuffer,
    pub mesh_bone_buffer: VirtualBuffer,

    pub skeleton_buffer: VirtualBuffer,
    pub skeleton_bone_buffer: VirtualBuffer,

    pub animation_buffer: VirtualBuffer,
    pub animation_frame_buffer: VirtualBuffer,

    pub material_buffer: VirtualBuffer,
}

impl ResourceBufferHandles {
    pub fn import(pass_graph: &mut PassGraph, resource_buffers: &ResourceBuffers) -> Self {
        Self {
            index_buffer: pass_graph.import_buffer(&resource_buffers.index.allocation),
            mesh_vertex_buffer: pass_graph.import_buffer(&resource_buffers.mesh_vertex.allocation),
            mesh_vertex_attribute_buffer: pass_graph.import_buffer(&resource_buffers.mesh_vertex_attribute.allocation),
            mesh_vertex_skin_buffer: pass_graph.import_buffer(&resource_buffers.mesh_vertex_skin.allocation),
            submesh_buffer: pass_graph.import_buffer(&resource_buffers.submesh.allocation),
            mesh_buffer: pass_graph.import_buffer(&resource_buffers.mesh.allocation),
            mesh_bone_buffer: pass_graph.import_buffer(&resource_buffers.mesh_bone.allocation),

            skeleton_buffer: pass_graph.import_buffer(&resource_buffers.skeleton.allocation),
            skeleton_bone_buffer: pass_graph.import_buffer(&resource_buffers.skeleton_bone.allocation),

            animation_buffer: pass_graph.import_buffer(&resource_buffers.animation.allocation),
            animation_frame_buffer: pass_graph.import_buffer(&resource_buffers.animation_frame.allocation),

            material_buffer: pass_graph.import_buffer(&resource_buffers.material.allocation),
        }
    }
}
