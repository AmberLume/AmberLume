use crate::resources::store::providers::mesh::buffer::mesh_buffer::MeshGPU;
use crate::resources::store::providers::mesh::buffer::submesh_buffer::SubmeshGPU;
use crate::resources::store::providers::resource_provider::ResourceId;

pub trait MeshLoadObserver: Send + Sync {
    fn on_load(&self, mesh_id: ResourceId, mesh: &MeshGPU, submeshes: &[SubmeshGPU]);
}
