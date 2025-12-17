use crate::data::adapter::mesh_adapter::MeshAdapter;
use crate::data::common::model_data;
use anyhow::Result;
use gltf::Document;
use gltf::buffer::Data;

pub struct ModelAdapter;

impl ModelAdapter {
    pub fn create_from(
        document: &Document,
        buffers: &[Data],
        name: String,
        mesh_indices: &[usize],
    ) -> Result<model_data::ModelData> {
        let document_meshes = document.meshes().collect::<Vec<_>>();

        let mut meshes = Vec::new();

        for mesh_index in mesh_indices {
            let mesh = &document_meshes[*mesh_index];

            let mesh_data = MeshAdapter::create_from(&mesh, &buffers)?;

            meshes.push(mesh_data);
        }

        Ok(model_data::ModelData {
            name,
            meshes,
            bounds: [0.0; 6],
        })
    }
}
