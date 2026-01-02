use crate::assembler::adapter::adapter::ResourceAdapter;
use anyhow::Result;
use gltf::{Document, Node};
use serde::Deserialize;
use crate::data::common::scene_data::{SceneData, SceneNodeData};

pub struct SceneAdapter;

#[derive(Deserialize, Debug, PartialEq)]
struct NodeExtras {
    pub asset_file_name: String,
    pub collection_name: String,
}

impl SceneAdapter {
    pub fn create() -> Self {
        Self {
            
        }
    }

    fn collect_node(nodes: &mut Vec<SceneNodeData>, node: Node) {
        let name = node.name()
            .expect("Node names are required! Parent: {}")
            .to_string();

        let mut asset_key = None;
        if let Some(raw_extras) = node.extras() {
            let raw_extras  = serde_json::from_str::<NodeExtras>(raw_extras.get());

            if let Ok(raw_extras) = raw_extras {
                asset_key = Some(format!("{}#{}", raw_extras.asset_file_name, raw_extras.collection_name));
            }
        }

        let (transform, rotation, scale) = node.transform().decomposed();

        for node in node.children() {
            Self::collect_node(nodes, node);
        }

        nodes.push(SceneNodeData {
            name,

            transform,
            rotation,
            scale,

            asset_key: asset_key.expect("Asset keys are required for nodes!"),
        })
    }
}

pub struct SceneResource {
    pub document: Document,
}

impl ResourceAdapter for SceneAdapter {
    type Input<'a> = SceneResource;

    type Output = Vec<SceneData>;

    fn adapt<'a>(&mut self, input: &Self::Input<'a>) -> Result<Self::Output> {
        let scenes: Vec<SceneData> = input.document.scenes()
            .map(|scene| {
                let name = scene.name().expect("Scene names are required").to_string();

                let mut nodes = Vec::new();

                for node in scene.nodes() {
                    Self::collect_node(&mut nodes, node);
                }

                SceneData {
                    name,

                    nodes,
                }
            })
            .collect();

        Ok(scenes)
    }
}
