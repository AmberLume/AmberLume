use gpu_data::MeshVertexAttributeGPU;
use gpu_data::MeshVertexGPU;
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
        let cube_vertices: Vec<MeshVertexGPU> = vec![
            MeshVertexGPU::new([-0.5, -0.5, 0.5], [0.0, 0.0, 1.0]),
            MeshVertexGPU::new([0.5, -0.5, 0.5], [0.0, 0.0, 1.0]),
            MeshVertexGPU::new([0.5, 0.5, 0.5], [0.0, 0.0, 1.0]),
            MeshVertexGPU::new([-0.5, 0.5, 0.5], [0.0, 0.0, 1.0]),
            // Back face (Z-)
            MeshVertexGPU::new([0.5, -0.5, -0.5], [0.0, 0.0, -1.0]),
            MeshVertexGPU::new([-0.5, -0.5, -0.5], [0.0, 0.0, -1.0]),
            MeshVertexGPU::new([-0.5, 0.5, -0.5], [0.0, 0.0, -1.0]),
            MeshVertexGPU::new([0.5, 0.5, -0.5], [0.0, 0.0, -1.0]),
            // Right face (X+)
            MeshVertexGPU::new([0.5, -0.5, 0.5], [1.0, 0.0, 0.0]),
            MeshVertexGPU::new([0.5, -0.5, -0.5], [1.0, 0.0, 0.0]),
            MeshVertexGPU::new([0.5, 0.5, -0.5], [1.0, 0.0, 0.0]),
            MeshVertexGPU::new([0.5, 0.5, 0.5], [1.0, 0.0, 0.0]),
            // Left face (X-)
            MeshVertexGPU::new([-0.5, -0.5, -0.5], [-1.0, 0.0, 0.0]),
            MeshVertexGPU::new([-0.5, -0.5, 0.5], [-1.0, 0.0, 0.0]),
            MeshVertexGPU::new([-0.5, 0.5, 0.5], [-1.0, 0.0, 0.0]),
            MeshVertexGPU::new([-0.5, 0.5, -0.5], [-1.0, 0.0, 0.0]),
            // Top face (Y+)
            MeshVertexGPU::new([-0.5, 0.5, 0.5], [0.0, 1.0, 0.0]),
            MeshVertexGPU::new([0.5, 0.5, 0.5], [0.0, 1.0, 0.0]),
            MeshVertexGPU::new([0.5, 0.5, -0.5], [0.0, 1.0, 0.0]),
            MeshVertexGPU::new([-0.5, 0.5, -0.5], [0.0, 1.0, 0.0]),
            // Bottom face (Y-)
            MeshVertexGPU::new([-0.5, -0.5, -0.5], [0.0, -1.0, 0.0]),
            MeshVertexGPU::new([0.5, -0.5, -0.5], [0.0, -1.0, 0.0]),
            MeshVertexGPU::new([0.5, -0.5, 0.5], [0.0, -1.0, 0.0]),
            MeshVertexGPU::new([-0.5, -0.5, 0.5], [0.0, -1.0, 0.0]),
        ];

        let cube_attributes: Vec<MeshVertexAttributeGPU> = vec![
            MeshVertexAttributeGPU::new(tangent, [0.0, 1.0]),
            MeshVertexAttributeGPU::new(tangent, [1.0, 1.0]),
            MeshVertexAttributeGPU::new(tangent, [1.0, 0.0]),
            MeshVertexAttributeGPU::new(tangent, [0.0, 0.0]),
            // Back face (Z-)
            MeshVertexAttributeGPU::new(tangent, [0.0, 1.0]),
            MeshVertexAttributeGPU::new(tangent, [1.0, 1.0]),
            MeshVertexAttributeGPU::new(tangent, [1.0, 0.0]),
            MeshVertexAttributeGPU::new(tangent, [0.0, 0.0]),
            // Right face (X+)
            MeshVertexAttributeGPU::new(tangent, [0.0, 1.0]),
            MeshVertexAttributeGPU::new(tangent, [1.0, 1.0]),
            MeshVertexAttributeGPU::new(tangent, [1.0, 0.0]),
            MeshVertexAttributeGPU::new(tangent, [0.0, 0.0]),
            // Left face (X-)
            MeshVertexAttributeGPU::new(tangent, [0.0, 1.0]),
            MeshVertexAttributeGPU::new(tangent, [1.0, 1.0]),
            MeshVertexAttributeGPU::new(tangent, [1.0, 0.0]),
            MeshVertexAttributeGPU::new(tangent, [0.0, 0.0]),
            // Top face (Y+)
            MeshVertexAttributeGPU::new(tangent, [0.0, 1.0]),
            MeshVertexAttributeGPU::new(tangent, [1.0, 1.0]),
            MeshVertexAttributeGPU::new(tangent, [1.0, 0.0]),
            MeshVertexAttributeGPU::new(tangent, [0.0, 0.0]),
            // Bottom face (Y-)
            MeshVertexAttributeGPU::new(tangent, [0.0, 1.0]),
            MeshVertexAttributeGPU::new(tangent, [1.0, 1.0]),
            MeshVertexAttributeGPU::new(tangent, [1.0, 0.0]),
            MeshVertexAttributeGPU::new(tangent, [0.0, 0.0]),
        ];

        let cube_submesh = vec![
            SubmeshConfig::new(
                cube_indices,
                cube_vertices,
                cube_attributes,
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
