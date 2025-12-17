use crate::data::adapter::primitive_adapter::PrimitiveAdapter;
use crate::data::common::mesh_data;
use anyhow::Result;
use gltf::Mesh;
use gltf::buffer::Data;

pub struct MeshAdapter;

impl MeshAdapter {
    pub fn create_from(mesh: &Mesh, buffers: &[Data]) -> Result<mesh_data::MeshData> {
        let name = mesh.name().expect("Mesh names are required").to_owned();

        let mut primitives = Vec::new();
        for primitive in mesh.primitives() {
            let primitive_data = PrimitiveAdapter::create_from(&primitive, buffers)?;

            primitives.push(primitive_data);
        }

        Ok(mesh_data::MeshData {
            name,
            primitives,
            bounds: [0.0; 6],
        })
    }
}
