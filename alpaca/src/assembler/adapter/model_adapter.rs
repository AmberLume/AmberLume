use std::path::Path;
use crate::assembler::aabb_utils::calculate_global_aabb;
use crate::assembler::adapter::adapter::ResourceAdapter;
use crate::assembler::adapter::mesh_adapter::{MeshAdapter, MeshResource};
use crate::data::common::model_data::ModelData;
use anyhow::Result;
use gltf::buffer::Data;
use gltf::{Mesh, Node};

pub struct ModelAdapter {
    mesh_adapter: MeshAdapter,
}

impl ModelAdapter {
    pub fn create(mesh_adapter: MeshAdapter) -> Self {
        Self { mesh_adapter }
    }

    fn collect_meshes<'a>(&self, node: &Node<'a>, meshes: &mut Vec<Mesh<'a>>) {
        if let Some(mesh) = node.mesh() {
            meshes.push(mesh);
        }

        for node in node.children() {
            self.collect_meshes(&node, meshes);
        }
    }
}

pub struct ModelResource<'a> {
    pub collection_node: Node<'a>,

    pub local_path: &'a Path,

    pub buffers: &'a [Data],
}

impl ResourceAdapter for ModelAdapter {
    type Input<'a> = ModelResource<'a>;

    type Output = ModelData;

    fn adapt<'a>(&mut self, input: &Self::Input<'a>) -> Result<Self::Output> {
        let mut meshes = Vec::new();
        let mut aabbs = Vec::new();

        self.collect_meshes(&input.collection_node, &mut meshes);

        let meshes = meshes.iter().map(|mesh| {
            let mesh_data = self.mesh_adapter.adapt(&MeshResource {
                mesh,

                local_path: input.local_path,

                buffers: input.buffers,
            }).unwrap();

            aabbs.push(mesh_data.bounds);

            mesh_data
        }).collect::<Vec<_>>();

        Ok(Self::Output {
            meshes,
            bounds: calculate_global_aabb(&aabbs),
        })
    }
}
