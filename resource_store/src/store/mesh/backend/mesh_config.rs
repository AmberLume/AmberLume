use crate::store::mesh::backend::submesh_config::SubmeshConfig;
use gpu_data::MeshBindingGPU;
use resource_residency::ResRef;
use std::sync::Arc;

#[derive(Clone)]
pub enum MeshConfig {
    Data {
        submeshes: Vec<SubmeshConfig>,
        bindings: Vec<MeshBindingGPU>,

        skeleton: Option<Arc<ResRef>>,
    },
    Instance {
        original: Arc<ResRef>,

        vertex_slice_count: u32,
        bounds: Option<[f32; 6]>,
    },
}
