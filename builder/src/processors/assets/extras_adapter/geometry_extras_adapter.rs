use anyhow::bail;
use anyhow::Result;
use buffer::Source;
use gltf::{Node, buffer};

#[derive(Debug, PartialEq)]
pub struct Geometry {
    pub positions: Vec<[f32; 3]>,
}

impl Geometry {
    pub fn adapt(node: &Node, bin: Option<&[u8]>) -> Result<Geometry> {
        let Some(mesh) = node.mesh() else {
            bail!("No geometry");
        };

        let mut positions = Vec::new();

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| match buffer.source() {
                Source::Bin => None,
                Source::Uri(_) => bin,
            });

            if let Some(values) = reader.read_positions() {
                positions.extend(values);
            }
        }

        Ok(Self {
            positions,
        })
    }
}
