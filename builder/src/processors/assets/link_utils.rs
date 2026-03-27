use gltf::Node;
use serde::Deserialize;
use serde_json::from_str;

#[derive(Deserialize, Debug, PartialEq)]
pub struct LinkExtras {
    pub source_gltf: String,
    pub source_collection: String,
}

impl LinkExtras {
    pub fn from_node(node: &Node) -> Option<LinkExtras> {
        node.extras()
            .as_ref()
            .and_then(|extras| from_str::<LinkExtras>(extras.get()).ok())
    }
}
