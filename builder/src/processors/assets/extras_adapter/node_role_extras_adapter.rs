use anyhow::{bail, Result};
use gltf::Node;
use serde::Deserialize;
use serde_json::from_str;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum NodeRoleType {
    Mesh,
    Collider,
    Placeholder,
    Skeleton,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct NodeRoleExtras {
    pub amberlume_type: NodeRoleType,
}

impl NodeRoleExtras {
    pub fn adapt(node: &Node) -> Result<NodeRoleExtras> {
        let Some(extras) = node.extras() else {
            bail!("No extras");
        };

        match from_str::<NodeRoleExtras>(extras.get()) {
            Ok(node_role) => Ok(node_role),
            Err(error) => bail!("Invalid type: {}", error),
        }
    }
}
