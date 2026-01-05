use crate::assembler::adapter::adapter::ResourceAdapter;
use anyhow::Result;
use gltf::{Document, Node};
use serde::Deserialize;
use serde_json::from_str;
use crate::data::common::scene_data::{ColliderType, SceneData, SceneNodeData, SceneNodeCollider, ColliderShape};

pub struct SceneAdapter;

#[derive(Deserialize, Debug, PartialEq)]
struct NodeExtras {
    pub asset_file_name: String,
    pub collection_name: String,

    #[serde(default)]
    colliders: Option<String>,
}

impl NodeExtras {
    pub fn get_colliders(&self) -> Option<Vec<NodeCollider>> {
        if let Some(colliders) = &self.colliders {
            Some(from_str::<Vec<NodeCollider>>(&colliders).unwrap())
        } else {
            None
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct NodeCollider {
    pub name: String,

    pub collider_type: NodeColliderType,
    pub shape: NodeColliderShape,

    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

#[derive(Deserialize, Debug, PartialEq)]
enum NodeColliderType {
    Static,
    Kinematic,
    Dynamic,
}

#[derive(Deserialize, Debug, PartialEq)]
enum NodeColliderShape {
    Box {
        size: [f32; 3],
    },
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
        let mut colliders = Vec::new();
        if let Some(raw_extras) = node.extras() {
            let raw_extras  = from_str::<NodeExtras>(raw_extras.get());

            if let Ok(raw_extras) = raw_extras {
                asset_key = Some(format!("{}#{}", raw_extras.asset_file_name, raw_extras.collection_name));

                colliders = if let Some(colliders) = raw_extras.get_colliders() {
                    colliders.iter().map(|collider| {
                        let collider_type = match collider.collider_type {
                            NodeColliderType::Static => ColliderType::Static,
                            NodeColliderType::Kinematic => ColliderType::Kinematic,
                            NodeColliderType::Dynamic => ColliderType::Dynamic,
                        };

                        let collider_shape = match collider.shape {
                            NodeColliderShape::Box { size } => { ColliderShape::Box { size } }
                        };

                        SceneNodeCollider {
                            collider_name: collider.name.clone(),
                            collider_type,
                            position: collider.position,
                            rotation: collider.rotation,
                            collider_shape,
                        }
                    }).collect()
                } else {
                    Vec::new()
                }
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

            colliders,
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
