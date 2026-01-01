use std::path::Path;
use crate::assembler::aabb_utils::{calculate_aabb, calculate_global_aabb};
use crate::assembler::adapter::adapter::ResourceAdapter;
use crate::assembler::adapter::primitive_adapter::{PrimitiveAdapter, PrimitiveResource};
use crate::data::common::mesh_data::MeshData;
use anyhow::Result;
use gltf::Mesh;
use gltf::buffer::Data;

pub struct MeshAdapter {
    primitive_adapter: PrimitiveAdapter,
}

impl MeshAdapter {
    pub fn create(primitive_adapter: PrimitiveAdapter) -> Self {
        Self { primitive_adapter }
    }
}

pub struct MeshResource<'a> {
    pub mesh: &'a Mesh<'a>,

    pub local_path: &'a Path,
    
    pub buffers: &'a [Data],
}

impl ResourceAdapter for MeshAdapter {
    type Input<'a> = MeshResource<'a>;
    type Output = MeshData;

    fn adapt<'a>(&mut self, input: &Self::Input<'a>) -> Result<Self::Output> {
        let name = input
            .mesh
            .name()
            .expect("Mesh names are required")
            .to_owned();
        println!("Collecting mesh '{}'...", name);

        let mut primitives = Vec::new();
        let mut aabbs = Vec::new();

        for primitive in input.mesh.primitives() {
            let primitive_data = self.primitive_adapter.adapt(&PrimitiveResource {
                primitive,

                local_path: input.local_path,
                
                buffers: &input.buffers,
            })?;
            let aabb = calculate_aabb(&primitive_data.vertices);

            aabbs.push(aabb);
            primitives.push(primitive_data);
        }

        Ok(Self::Output {
            name,
            primitives,
            bounds: calculate_global_aabb(&aabbs),
        })
    }
}
