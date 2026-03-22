use gltf::Node;
use serde::Deserialize;
use serde_json::from_str;
use tracing::error;

#[derive(Deserialize, Debug, PartialEq)]
pub struct LinkExtras {
    pub source_gltf: String,
    pub source_collection: String,
}

pub fn extract_link_extras(mesh_node: &Node) -> Option<LinkExtras> {
    let extras = mesh_node
        .extras()
        .as_ref()
        .and_then(|extras| from_str::<LinkExtras>(extras.get()).ok());

    if extras.is_none() {
        error!(
            "Failed to extract LinkExtras. Extras: {:?}",
            mesh_node.extras()
        );
    }

    extras
}
