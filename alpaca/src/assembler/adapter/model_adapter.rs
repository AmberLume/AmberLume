use crate::assembler::aabb_utils::calculate_global_aabb;
use crate::assembler::adapter::adapter::ResourceAdapter;
use crate::assembler::adapter::mesh_adapter::{MeshAdapter, MeshResource};
use crate::data::common::model_data::ModelData;
use anyhow::Result;
use gltf::buffer::Data;
use gltf::{Document, Node};

pub struct ModelAdapter {
    mesh_adapter: MeshAdapter,
}

impl ModelAdapter {
    pub fn create(mesh_adapter: MeshAdapter) -> Self {
        Self { mesh_adapter }
    }

    fn collect_node(node: &Node, meshes: &mut Vec<usize>) {
        if let Some(mesh) = node.mesh() {
            let index = mesh.index();

            meshes.push(index);
        }

        for node in node.children() {
            Self::collect_node(&node, meshes)
        }
    }
}

pub struct ModelResource<'a> {
    pub document: Document,

    pub buffers: &'a [Data],
}

impl ResourceAdapter for ModelAdapter {
    type Input<'a> = ModelResource<'a>;

    type Output = ModelData;

    fn adapt<'a>(&mut self, input: &Self::Input<'a>) -> Result<Self::Output> {
        let mut scene_meshes = Vec::new();

        for scene in input.document.scenes() {
            for node in scene.nodes() {
                Self::collect_node(&node, &mut scene_meshes);
            }
        }

        scene_meshes.sort();
        scene_meshes.dedup();

        let mut meshes = Vec::new();
        let mut aabbs = Vec::new();

        for mesh_index in scene_meshes {
            let mesh_data = self.mesh_adapter.adapt(&MeshResource {
                mesh: input.document.meshes().nth(mesh_index).unwrap(),

                buffers: input.buffers,
            })?;

            aabbs.push(mesh_data.bounds);
            meshes.push(mesh_data);
        }

        Ok(Self::Output {
            meshes,
            bounds: calculate_global_aabb(&aabbs),
        })
    }
}
