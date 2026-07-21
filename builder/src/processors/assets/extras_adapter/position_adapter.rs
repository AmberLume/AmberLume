use anyhow::Result;
use gltf::Node;

#[derive(Debug, PartialEq)]
pub struct Position {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Position {
    pub fn adapt(node: &Node) -> Result<Position> {
        let (translation, rotation, scale) = node.transform().decomposed();

        Ok(Self {
            translation,
            rotation,
            scale,
        })
    }
}
