use std::sync::Arc;
use anyhow::Result;
use glam::{vec2, vec3};
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::buffer::typed::model_buffer::ModelGpuData;
use crate::render::vulkan::buffer::typed::submesh_buffer::SubmeshGpuData;
use crate::render::vulkan::buffer::typed::vertex_buffer::VertexGpuData;
use crate::render::vulkan::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_manager::DescriptorIndexManager;
use crate::resources::dynamic::resource_provider::ResourceId;
use crate::resources::persistent::persistent_materials::PersistentMaterials;

pub struct PersistentModels {
    pub cube: (ResourceId, ModelGpuData),
}

impl PersistentModels {
    pub fn create(
        resource_loader: Arc<ResourceLoader>,
        persistent_materials: &PersistentMaterials,
        index_index_manager: &DescriptorIndexManager,
        vertex_index_manager: &DescriptorIndexManager,
        submesh_index_manager: &DescriptorIndexManager,
        model_index_manager: &DescriptorIndexManager,
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
            &buffer_manager.index_buffer,
            first_index_resource_id,
            &cube_indices
        )?;

        let cube_vertices: Vec<VertexGpuData> = vec![
            VertexGpuData::create(vec3(-0.5, -0.5,  0.5), vec3(0.0, 0.0, 1.0), vec2(0.0, 1.0)),
            VertexGpuData::create(vec3( 0.5, -0.5,  0.5), vec3(0.0, 0.0, 1.0), vec2(1.0, 1.0)),
            VertexGpuData::create(vec3( 0.5,  0.5,  0.5), vec3(0.0, 0.0, 1.0), vec2(1.0, 0.0)),
            VertexGpuData::create(vec3(-0.5,  0.5,  0.5), vec3(0.0, 0.0, 1.0), vec2(0.0, 0.0)),
            // Back face (Z-)
            VertexGpuData::create(vec3( 0.5, -0.5, -0.5), vec3(0.0, 0.0, -1.0), vec2(0.0, 1.0)),
            VertexGpuData::create(vec3(-0.5, -0.5, -0.5), vec3(0.0, 0.0, -1.0), vec2(1.0, 1.0)),
            VertexGpuData::create(vec3(-0.5,  0.5, -0.5), vec3(0.0, 0.0, -1.0), vec2(1.0, 0.0)),
            VertexGpuData::create(vec3( 0.5,  0.5, -0.5), vec3(0.0, 0.0, -1.0), vec2(0.0, 0.0)),
            // Right face (X+)
            VertexGpuData::create(vec3( 0.5, -0.5,  0.5), vec3(1.0, 0.0, 0.0), vec2(0.0, 1.0)),
            VertexGpuData::create(vec3( 0.5, -0.5, -0.5), vec3(1.0, 0.0, 0.0), vec2(1.0, 1.0)),
            VertexGpuData::create(vec3( 0.5,  0.5, -0.5), vec3(1.0, 0.0, 0.0), vec2(1.0, 0.0)),
            VertexGpuData::create(vec3( 0.5,  0.5,  0.5), vec3(1.0, 0.0, 0.0), vec2(0.0, 0.0)),
            // Left face (X-)
            VertexGpuData::create(vec3(-0.5, -0.5, -0.5), vec3(-1.0, 0.0, 0.0), vec2(0.0, 1.0)),
            VertexGpuData::create(vec3(-0.5, -0.5,  0.5), vec3(-1.0, 0.0, 0.0), vec2(1.0, 1.0)),
            VertexGpuData::create(vec3(-0.5,  0.5,  0.5), vec3(-1.0, 0.0, 0.0), vec2(1.0, 0.0)),
            VertexGpuData::create(vec3(-0.5,  0.5, -0.5), vec3(-1.0, 0.0, 0.0), vec2(0.0, 0.0)),
            // Top face (Y+)
            VertexGpuData::create(vec3(-0.5,  0.5,  0.5), vec3(0.0, 1.0, 0.0), vec2(0.0, 1.0)),
            VertexGpuData::create(vec3( 0.5,  0.5,  0.5), vec3(0.0, 1.0, 0.0), vec2(1.0, 1.0)),
            VertexGpuData::create(vec3( 0.5,  0.5, -0.5), vec3(0.0, 1.0, 0.0), vec2(1.0, 0.0)),
            VertexGpuData::create(vec3(-0.5,  0.5, -0.5), vec3(0.0, 1.0, 0.0), vec2(0.0, 0.0)),
            // Bottom face (Y-)
            VertexGpuData::create(vec3(-0.5, -0.5, -0.5), vec3(0.0, -1.0, 0.0), vec2(0.0, 1.0)),
            VertexGpuData::create(vec3( 0.5, -0.5, -0.5), vec3(0.0, -1.0, 0.0), vec2(1.0, 1.0)),
            VertexGpuData::create(vec3( 0.5, -0.5,  0.5), vec3(0.0, -1.0, 0.0), vec2(1.0, 0.0)),
            VertexGpuData::create(vec3(-0.5, -0.5,  0.5), vec3(0.0, -1.0, 0.0), vec2(0.0, 0.0)),
        ];
        let first_vertex_resource_id = vertex_index_manager.acquire_range(cube_vertices.len() as u32).unwrap();
        resource_loader.load_buffer_at(
            &buffer_manager.vertex_buffer,
            first_vertex_resource_id,
            &cube_vertices,
        )?;

        let cube_submeshes = [
            SubmeshGpuData::create(
                cube_indices.len() as u32,
                first_index_resource_id as u32,
                first_vertex_resource_id as u32,
                persistent_materials.default.0,
                [-0.5, -0.5, -0.5, 0.5, 0.5, 0.5],
            )
        ];
        let first_submesh_resource_id = submesh_index_manager.acquire_range(cube_submeshes.len() as u32).unwrap();
        resource_loader.load_buffer_at(
            &buffer_manager.submesh_buffer,
            first_submesh_resource_id,
            &cube_submeshes,
        )?;

        let cube_model = ModelGpuData::create(
            first_submesh_resource_id as u32,
            cube_submeshes.len() as u32,
        );
        let model_resource_id = model_index_manager.acquire_range(1).unwrap();
        resource_loader.load_buffer_at(
            &buffer_manager.model_buffer,
            model_resource_id,
            &[cube_model],
        )?;

        Ok(Self {
            cube: (model_resource_id, cube_model),
        })
    }
}
