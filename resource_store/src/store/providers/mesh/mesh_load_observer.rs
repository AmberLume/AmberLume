use gpu_data::MeshGPU;
use gpu_data::SubmeshGPU;
use index_allocator::ResourceId;

pub trait MeshLoadObserver: Send + Sync {
    fn on_load(&self, mesh_id: ResourceId, mesh: &MeshGPU, submeshes: &[SubmeshGPU]);

    fn on_unload(&self, mesh_id: ResourceId);
}
