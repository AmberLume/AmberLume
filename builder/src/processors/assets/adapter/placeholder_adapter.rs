use crate::processors::assets::extras_adapter::placeholder_extras_adapter::PlaceholderExtras;
use crate::processors::assets::extras_adapter::position_adapter::Position;
use anyhow::Result;
use gltf::Node;
use resource_data::scene_data::BodyTypeData;

#[derive(Debug, PartialEq)]
pub struct Placeholder {
    pub body_type: BodyTypeData,

    pub source_gltf: String,
    pub source_collection: String,

    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Placeholder {
    pub fn adapt(node: &Node) -> Result<Placeholder> {
        let placeholder_extras = PlaceholderExtras::adapt(node)?;
        let position = Position::adapt(node)?;

        Ok(Self {
            body_type: placeholder_extras.body_type,

            source_gltf: placeholder_extras.source_gltf,
            source_collection: placeholder_extras.source_collection,

            translation: position.translation,
            rotation: position.rotation,
            scale: position.scale,
        })
    }
}
