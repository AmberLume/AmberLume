use std::sync::Arc;
use bytemuck::cast_slice;
use crate::render::buffer::typed::vertex_buffer::VertexGPU;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_backend::ResourceKey;
use crate::resources::utils::hasher::hasher::Hasher;

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

impl MeshConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        match self {
            Self::Alpaca { resource_key: asset_key } => {
                hasher.hash_u32(0);

                hasher.hash_string(&asset_key);
            }
            Self::InBuilt {
                submeshes,
                skeleton,
            } => {
                hasher.hash_u32(1);

                hasher.hash_u32(submeshes.len() as u32);
                for submesh in submeshes {
                    hasher.hash_u32(submesh.indices.len() as u32);
                    for index in &submesh.indices {
                        hasher.hash_u32(*index);
                    }

                    hasher.hash_u32(submesh.vertices.len() as u32);
                    for vertex in &submesh.vertices {
                        hasher.hash_u8_slice(cast_slice(&vertex.position));
                        hasher.hash_u8_slice(cast_slice(&vertex.normal));
                        hasher.hash_u8_slice(cast_slice(&vertex.tangent));
                        hasher.hash_u8_slice(cast_slice(&vertex.uv));
                    }

                    hasher.hash_u32(submesh.material.id);
                    hasher.hash_u8_slice(cast_slice(&submesh.aabb));
                }

                if let Some(skeleton) = &skeleton {
                    hasher.hash_u32(0);

                    hasher.hash_u32(skeleton.id)
                } else {
                    hasher.hash_u32(1);
                }
            }
        }

        hasher.finalize()
    }
}
