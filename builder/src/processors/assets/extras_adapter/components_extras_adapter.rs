use anyhow::bail;
use anyhow::Result;
use gltf::Node;
use resource_data::component_data::ComponentData;
use serde::Deserialize;
use serde_json::from_str;

#[derive(Deserialize, Debug, PartialEq)]
pub struct ComponentsExtras {
    #[serde(default)]
    pub amberlume_components: Vec<ComponentData>,
}

impl ComponentsExtras {
    pub fn adapt(node: &Node) -> Result<ComponentsExtras> {
        let Some(extras) = node.extras() else {
            bail!("No extras");
        };

        match from_str::<ComponentsExtras>(extras.get()) {
            Ok(extras) => Ok(extras),
            Err(error) => bail!("Invalid components: {}", error),
        }
    }
}
