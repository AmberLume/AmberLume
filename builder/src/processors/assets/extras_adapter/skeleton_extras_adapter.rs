use anyhow::bail;
use anyhow::Result;
use gltf::Node;
use serde::Deserialize;
use serde_json::from_str;

#[derive(Deserialize, Debug, PartialEq)]
pub struct SkeletonExtras {
    pub source_gltf: Option<String>,
}

impl SkeletonExtras {
    pub fn adapt(node: &Node) -> Result<SkeletonExtras> {
        let Some(extras) = node.extras() else {
            bail!("No extras");
        };

        match from_str::<SkeletonExtras>(extras.get()) {
            Ok(extras) => Ok(extras),
            Err(error) => bail!("Invalid extras: {}", error),
        }
    }
}
