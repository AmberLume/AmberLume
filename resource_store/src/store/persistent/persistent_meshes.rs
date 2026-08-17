use gpu_data::VertexGPU;
use std::sync::Arc;
use anyhow::Result;
use crate::store::persistent::persistent_materials::PersistentMaterials;
use resource_residency::ResRef;
use resource_residency::ResourceProvider;
use crate::store::providers::mesh::mesh_backend::MeshBackend;
use crate::store::providers::mesh::mesh_config::{MeshConfig, SubmeshConfig};

pub struct PersistentMeshes {
    pub cube: Arc<ResRef>,
}

impl PersistentMeshes {
    pub fn create(
        meshes_provider: &ResourceProvider<MeshBackend>,
        persistent_materials: &PersistentMaterials,
    ) -> Result<Self> {
        let cube_indices: Vec<u32> = vec![
            0,  1,  2,  2,  3,  0,  // Front
            4,  5,  6,  6,  7,  4,  // Back
            8,  9,  10, 10, 11, 8,  // Right
            12, 13, 14, 14, 15, 12, // Left
            16, 17, 18, 18, 19, 16, // Top
            20, 21, 22, 22, 23, 20, // Bottom
        ];

        let tangent = [1.0, 1.0, 1.0, 1.0];
        let bone_indices = [0; 2];
        let bone_weights = [0.0; 4];
        let cube_vertices: Vec<VertexGPU> = vec![
            VertexGPU::new([-0.5, -0.5, 0.5], [0.0, 0.0, 1.0], tangent, [0.0, 1.0], bone_indices, bone_weights),
            VertexGPU::new([0.5, -0.5, 0.5], [0.0, 0.0, 1.0], tangent, [1.0, 1.0], bone_indices, bone_weights),
            VertexGPU::new([0.5, 0.5, 0.5], [0.0, 0.0, 1.0], tangent, [1.0, 0.0], bone_indices, bone_weights),
            VertexGPU::new([-0.5, 0.5, 0.5], [0.0, 0.0, 1.0], tangent, [0.0, 0.0], bone_indices, bone_weights),
            // Back face (Z-)
            VertexGPU::new([0.5, -0.5, -0.5], [0.0, 0.0, -1.0], tangent, [0.0, 1.0], bone_indices, bone_weights),
            VertexGPU::new([-0.5, -0.5, -0.5], [0.0, 0.0, -1.0], tangent, [1.0, 1.0], bone_indices, bone_weights),
            VertexGPU::new([-0.5, 0.5, -0.5], [0.0, 0.0, -1.0], tangent, [1.0, 0.0], bone_indices, bone_weights),
            VertexGPU::new([0.5, 0.5, -0.5], [0.0, 0.0, -1.0], tangent, [0.0, 0.0], bone_indices, bone_weights),
            // Right face (X+)
            VertexGPU::new([0.5, -0.5, 0.5], [1.0, 0.0, 0.0], tangent, [0.0, 1.0], bone_indices, bone_weights),
            VertexGPU::new([0.5, -0.5, -0.5], [1.0, 0.0, 0.0], tangent, [1.0, 1.0], bone_indices, bone_weights),
            VertexGPU::new([0.5, 0.5, -0.5], [1.0, 0.0, 0.0], tangent, [1.0, 0.0], bone_indices, bone_weights),
            VertexGPU::new([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], tangent, [0.0, 0.0], bone_indices, bone_weights),
            // Left face (X-)
            VertexGPU::new([-0.5, -0.5, -0.5], [-1.0, 0.0, 0.0], tangent, [0.0, 1.0], bone_indices, bone_weights),
            VertexGPU::new([-0.5, -0.5, 0.5], [-1.0, 0.0, 0.0], tangent, [1.0, 1.0], bone_indices, bone_weights),
            VertexGPU::new([-0.5, 0.5, 0.5], [-1.0, 0.0, 0.0], tangent, [1.0, 0.0], bone_indices, bone_weights),
            VertexGPU::new([-0.5, 0.5, -0.5], [-1.0, 0.0, 0.0], tangent, [0.0, 0.0], bone_indices, bone_weights),
            // Top face (Y+)
            VertexGPU::new([-0.5, 0.5, 0.5], [0.0, 1.0, 0.0], tangent, [0.0, 1.0], bone_indices, bone_weights),
            VertexGPU::new([0.5, 0.5, 0.5], [0.0, 1.0, 0.0], tangent, [1.0, 1.0], bone_indices, bone_weights),
            VertexGPU::new([0.5, 0.5, -0.5], [0.0, 1.0, 0.0], tangent, [1.0, 0.0], bone_indices, bone_weights),
            VertexGPU::new([-0.5, 0.5, -0.5], [0.0, 1.0, 0.0], tangent, [0.0, 0.0], bone_indices, bone_weights),
            // Bottom face (Y-)
            VertexGPU::new([-0.5, -0.5, -0.5], [0.0, -1.0, 0.0], tangent, [0.0, 1.0], bone_indices, bone_weights),
            VertexGPU::new([0.5, -0.5, -0.5], [0.0, -1.0, 0.0], tangent, [1.0, 1.0], bone_indices, bone_weights),
            VertexGPU::new([0.5, -0.5, 0.5], [0.0, -1.0, 0.0], tangent, [1.0, 0.0], bone_indices, bone_weights),
            VertexGPU::new([-0.5, -0.5, 0.5], [0.0, -1.0, 0.0], tangent, [0.0, 0.0], bone_indices, bone_weights),
        ];

        let cube_submesh = vec![
            SubmeshConfig::new(
                cube_indices,
                cube_vertices,
                persistent_materials.default.clone(),
                [-0.5, -0.5, -0.5, 0.5, 0.5, 0.5],
            )
        ];

        let cube = meshes_provider.get_or_load(MeshConfig::InBuilt {
            submeshes: cube_submesh,
            skeleton: None,
        })?;

        Ok(Self {
            cube,
        })
    }

    pub fn destroy(self) {
        drop(self.cube);
    }
}
