use anyhow::Result;
use gltf::Node;
use crate::processors::assets::adapter::submesh_adapter::Submesh;

#[derive(Debug, PartialEq)]
pub struct Mesh {
    pub submeshes: Vec<Submesh>,
}

impl Mesh {
    pub fn adapt(node: &Node, bin: Option<&[u8]>) -> Result<Mesh> {
        let mut submeshes = Vec::new();

        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                submeshes.push(Submesh::adapt(&primitive, bin, &[], &[])?);
            }
        }

        Ok(Self {
            submeshes,
        })
    }
}
