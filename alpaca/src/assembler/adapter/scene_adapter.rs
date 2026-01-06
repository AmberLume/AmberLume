use crate::assembler::adapter::adapter::ResourceAdapter;
use anyhow::{bail, Result};
use gltf::{Document, Node};
use serde::Deserialize;
use serde_json::from_str;
use crate::data::common::scene_data::{SceneData, SceneNodeData, SceneNodeColliderData, ColliderShapeData, BodyTypeData, PhysicalBodyData};

pub struct SceneAdapter;

#[derive(Deserialize, Debug, PartialEq)]
struct NodeExtras {
    pub asset_file_name: String,
    pub collection_name: String,

    pub physical_body: PhysicalBody,
}

#[derive(Deserialize, Debug, PartialEq)]
struct PhysicalBody {
    pub body_type: NodeBodyType,

    pub colliders: Vec<NodeCollider>,
}

#[derive(Deserialize, Debug, PartialEq)]
enum NodeBodyType {
    Static,
    Kinematic,
    Dynamic,
}

#[derive(Deserialize, Debug, PartialEq)]
struct NodeCollider {
    pub name: String,

    pub shape: NodeColliderShape,

    pub position: [f32; 3],
    pub rotation: [f32; 4],
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

    fn collect_node(nodes: &mut Vec<SceneNodeData>, node: Node) -> Result<()> {
        let name = node.name()
            .expect("Node names are required! Parent: {}")
            .to_string();
        
        let raw_extras = if let Some(raw) = node.extras() {
            raw
        } else {
            bail!("Extras are required!")
        };
        
        let node_extras = from_str::<NodeExtras>(raw_extras.get()).expect("NodeExtras are invalid!");
        
        let asset_key = format!("{}#{}", node_extras.asset_file_name, node_extras.collection_name);
        let physical_body = Self::create_physical_body(node_extras.physical_body);

        let (transform, rotation, scale) = node.transform().decomposed();

        for node in node.children() {
            Self::collect_node(nodes, node)?;
        }

        nodes.push(SceneNodeData {
            name,

            transform,
            rotation,
            scale,

            asset_key,

            physical_body,
        });
        
        Ok(())
    }
    
    fn create_physical_body(physical_body: PhysicalBody) -> PhysicalBodyData {
        let body_type = match physical_body.body_type {
            NodeBodyType::Static => BodyTypeData::Static,
            NodeBodyType::Kinematic => BodyTypeData::Kinematic,
            NodeBodyType::Dynamic => BodyTypeData::Dynamic,
        };

        let colliders = physical_body.colliders.iter().map(|collider| {
            let collider_shape = match collider.shape {
                NodeColliderShape::Box { size } => { ColliderShapeData::Box { size } }
            };

            SceneNodeColliderData {
                collider_name: collider.name.clone(),

                collider_shape,

                position: collider.position,
                rotation: collider.rotation,
            }
        }).collect();
            
        PhysicalBodyData {
            body_type,
            
            colliders,
        }
    }
}

pub struct SceneResource {
    pub document: Document,
}

impl ResourceAdapter for SceneAdapter {
    type Input<'a> = SceneResource;

    type Output = Vec<SceneData>;

    fn adapt<'a>(&mut self, input: &Self::Input<'a>) -> Result<Self::Output> {
        let mut scenes: Vec<SceneData> = Vec::new();
        
        for scene in input.document.scenes() {
            let name = scene.name().expect("Scene names are required").to_string();

            let mut nodes = Vec::new();

            for node in scene.nodes() {
                Self::collect_node(&mut nodes, node)?;
            }

            scenes.push(SceneData {
                name,

                nodes,
            })
        }

        Ok(scenes)
    }
}
