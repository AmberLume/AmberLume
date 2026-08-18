use gpu_data::VertexGPU;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use resource_residency::ResRef;

#[derive(Clone)]
pub enum MeshConfig {
    Alpaca {
        resource_key: String,
    },
    InBuilt {
        submeshes: Vec<SubmeshConfig>,
        skeleton: Option<Arc<ResRef>>,
    },
    Reserved {
        key: u64,
        vertex_count: u32,
        index_offset: u32,
        index_count: u32,
        material: Arc<ResRef>,
        bounds: [f32; 6],
    }
}

impl Hash for MeshConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            MeshConfig::Alpaca { 
                resource_key,
            } => {
                0.hash(state);
                
                resource_key.hash(state);
            }
            MeshConfig::InBuilt { 
                submeshes,
                skeleton,
            } => {
                1.hash(state);

                submeshes.hash(state);
                skeleton.as_ref().map(|r| r.id).hash(state);
            }
            MeshConfig::Reserved {
                key,
                vertex_count,
                index_offset,
                index_count,
                material,
                bounds,
            } => {
                2.hash(state);

                key.hash(state);
                vertex_count.hash(state);
                index_offset.hash(state);
                index_count.hash(state);
                material.id.hash(state);

                for value in bounds {
                    value.to_bits().hash(state);
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct SubmeshConfig {
    pub indices: Vec<u32>,
    pub vertices: Vec<VertexGPU>,
    pub material: Arc<ResRef>,
    pub aabb: [f32; 6],
}

impl SubmeshConfig {
    pub fn new(
        indices: Vec<u32>,
        vertices: Vec<VertexGPU>,
        material: Arc<ResRef>,
        aabb: [f32; 6],
    ) -> Self {
        Self {
            indices,
            vertices,
            material,
            aabb,
        }
    }
}

impl Hash for SubmeshConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self {
            indices,
            vertices,
            material,
            aabb,
        } = self;

        indices.hash(state);
        vertices.hash(state);
        material.id.hash(state);

        for value in aabb {
            value.to_bits().hash(state);
        }
    }
}
