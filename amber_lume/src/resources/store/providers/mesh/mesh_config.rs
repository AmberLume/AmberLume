use std::hash::{Hash, Hasher};
use std::sync::Arc;
use crate::resources::store::providers::mesh::buffer::vertex_buffer::VertexGPU;
use crate::resources::store::providers::res_ref::ResRef;

#[derive(Clone)]
pub enum MeshConfig {
    Alpaca {
        resource_key: String,
    },
    InBuilt {
        submeshes: Vec<SubmeshConfig>,
        skeleton: Option<Arc<ResRef>>,
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
