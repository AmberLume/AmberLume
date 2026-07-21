use anyhow::bail;
use anyhow::Result;
use gltf::Node;
use resource_data::scene_data::BodyTypeData;
use serde::Deserialize;
use serde_json::from_str;

#[derive(Deserialize, Debug, PartialEq)]
pub struct PlaceholderExtras {
    pub body_type: BodyTypeData,

    pub source_gltf: String,
    pub source_collection: String,
}

impl PlaceholderExtras {
    pub fn adapt(node: &Node) -> Result<PlaceholderExtras> {
        let Some(extras) = node.extras() else {
            bail!("No extras");
        };

        match from_str::<PlaceholderExtras>(extras.get()) {
            Ok(extras) => Ok(extras),
            Err(error) => bail!("Invalid extras: {}", error),
        }
    }
}
