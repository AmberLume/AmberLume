use std::sync::Arc;
use anyhow::Result;
use glam::{vec2, vec3, vec4};
use crate::ids::SliceIndex;
use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::buffer::typed::mesh_buffer::MeshGPU;
use crate::render::buffer::typed::submesh_buffer::SubmeshGPU;
use crate::render::buffer::typed::vertex_buffer::VertexGPU;
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_manager::IndexManager;
use crate::resources::dynamic::resource_provider::ResourceId;
use crate::resources::persistent::persistent_materials::PersistentMaterials;

pub struct PersistentMeshes {
    pub cube: (ResourceId, MeshGPU),
}

impl PersistentMeshes {
    pub fn create(
        resource_loader: Arc<ResourceLoader>,
        persistent_materials: &PersistentMaterials,
        index_index_manager: &IndexManager,
        vertex_index_manager: &IndexManager,
        mesh_index_manager: &IndexManager,
        submesh_index_manager: &IndexManager,
        buffer_manager: &BufferManager,
    ) -> Result<Self> {
        let cube_indices: Vec<u32> = vec![
            0,  1,  2,  2,  3,  0,  // Front
            4,  5,  6,  6,  7,  4,  // Back
            8,  9,  10, 10, 11, 8,  // Right
            12, 13, 14, 14, 15, 12, // Left
            16, 17, 18, 18, 19, 16, // Top
            20, 21, 22, 22, 23, 20, // Bottom
        ];
        let first_index_resource_id = index_index_manager.acquire_range(cube_indices.len() as u32).unwrap();
        resource_loader.load_buffer_at(
            &buffer_manager.index_buffer.slice_at(SliceIndex { value: first_index_resource_id }),
            &cube_indices
        )?;

        let cube_vertices: Vec<VertexGPU> = vec![
            VertexGPU::create(vec3(-0.5, -0.5, 0.5), vec3(0.0, 0.0, 1.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(0.0, 1.0)),
            VertexGPU::create(vec3(0.5, -0.5, 0.5), vec3(0.0, 0.0, 1.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(1.0, 1.0)),
            VertexGPU::create(vec3(0.5, 0.5, 0.5), vec3(0.0, 0.0, 1.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(1.0, 0.0)),
            VertexGPU::create(vec3(-0.5, 0.5, 0.5), vec3(0.0, 0.0, 1.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(0.0, 0.0)),
            // Back face (Z-)
            VertexGPU::create(vec3(0.5, -0.5, -0.5), vec3(0.0, 0.0, -1.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(0.0, 1.0)),
            VertexGPU::create(vec3(-0.5, -0.5, -0.5), vec3(0.0, 0.0, -1.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(1.0, 1.0)),
            VertexGPU::create(vec3(-0.5, 0.5, -0.5), vec3(0.0, 0.0, -1.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(1.0, 0.0)),
            VertexGPU::create(vec3(0.5, 0.5, -0.5), vec3(0.0, 0.0, -1.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(0.0, 0.0)),
            // Right face (X+)
            VertexGPU::create(vec3(0.5, -0.5, 0.5), vec3(1.0, 0.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(0.0, 1.0)),
            VertexGPU::create(vec3(0.5, -0.5, -0.5), vec3(1.0, 0.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(1.0, 1.0)),
            VertexGPU::create(vec3(0.5, 0.5, -0.5), vec3(1.0, 0.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(1.0, 0.0)),
            VertexGPU::create(vec3(0.5, 0.5, 0.5), vec3(1.0, 0.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(0.0, 0.0)),
            // Left face (X-)
            VertexGPU::create(vec3(-0.5, -0.5, -0.5), vec3(-1.0, 0.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(0.0, 1.0)),
            VertexGPU::create(vec3(-0.5, -0.5, 0.5), vec3(-1.0, 0.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(1.0, 1.0)),
            VertexGPU::create(vec3(-0.5, 0.5, 0.5), vec3(-1.0, 0.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(1.0, 0.0)),
            VertexGPU::create(vec3(-0.5, 0.5, -0.5), vec3(-1.0, 0.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(0.0, 0.0)),
            // Top face (Y+)
            VertexGPU::create(vec3(-0.5, 0.5, 0.5), vec3(0.0, 1.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(0.0, 1.0)),
            VertexGPU::create(vec3(0.5, 0.5, 0.5), vec3(0.0, 1.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(1.0, 1.0)),
            VertexGPU::create(vec3(0.5, 0.5, -0.5), vec3(0.0, 1.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(1.0, 0.0)),
            VertexGPU::create(vec3(-0.5, 0.5, -0.5), vec3(0.0, 1.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(0.0, 0.0)),
            // Bottom face (Y-)
            VertexGPU::create(vec3(-0.5, -0.5, -0.5), vec3(0.0, -1.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(0.0, 1.0)),
            VertexGPU::create(vec3(0.5, -0.5, -0.5), vec3(0.0, -1.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(1.0, 1.0)),
            VertexGPU::create(vec3(0.5, -0.5, 0.5), vec3(0.0, -1.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(1.0, 0.0)),
            VertexGPU::create(vec3(-0.5, -0.5, 0.5), vec3(0.0, -1.0, 0.0), vec4(1.0, 1.0, 1.0, 1.0), vec2(0.0, 0.0)),
        ];
        let first_vertex_resource_id = vertex_index_manager.acquire_range(cube_vertices.len() as u32).unwrap();
        resource_loader.load_buffer_at(
            &buffer_manager.vertex_buffer.slice_at(SliceIndex { value: first_vertex_resource_id }),
            &cube_vertices,
        )?;

        let cube_submeshes = [
            SubmeshGPU::create(
                cube_indices.len() as u32,
                first_index_resource_id as u32,
                first_vertex_resource_id as u32,
                persistent_materials.default.0,
                [-0.5, -0.5, -0.5, 0.5, 0.5, 0.5],
            )
        ];
        let first_submesh_resource_id = submesh_index_manager.acquire_range(cube_submeshes.len() as u32).unwrap();
        resource_loader.load_buffer_at(
            &buffer_manager.submesh_buffer.slice_at(SliceIndex { value: first_submesh_resource_id }),
            &cube_submeshes,
        )?;

        let cube_mesh = MeshGPU::create(
            first_submesh_resource_id as u32,
            cube_submeshes.len() as u32,
        );
        let mesh_resource_id = mesh_index_manager.acquire_range(1).unwrap();
        resource_loader.load_buffer_at(
            &buffer_manager.mesh_buffer.slice_at(SliceIndex { value: mesh_resource_id }),
            &[cube_mesh],
        )?;

        Ok(Self {
            cube: (mesh_resource_id, cube_mesh),
        })
    }
}
