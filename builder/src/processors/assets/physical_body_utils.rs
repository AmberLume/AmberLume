use crate::build_task::{BuildTask, WriteFileTask};
use crate::data::physical_body_data::{ColliderData, ColliderShape, PhysicalBodyData};
use crate::dispatcher::Dispatcher;
use crate::paths::Paths;
use anyhow::Result;
use gltf::{Node, buffer};
use rkyv::rancor::Error;
use rkyv::to_bytes;
use serde::Deserialize;
use serde_json::from_str;
use std::sync::Arc;
use tracing::error;

#[derive(Deserialize, Debug, PartialEq)]
pub enum ColliderShapeType {
    Box,
    ConvexHull,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ColliderExtras {
    pub collider_name: String,
    pub collider_shape: ColliderShapeType,
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
}

fn extract_collider_extras(mesh_node: &Node) -> Option<ColliderExtras> {
    let extras = mesh_node
        .extras()
        .as_ref()
        .and_then(|extras| from_str::<ColliderExtras>(extras.get()).ok());

    if extras.is_none() {
        error!(
            "Failed to extract ColliderExtras. Extras: {:?}",
            mesh_node.extras()
        );
    }

    extras
}

pub fn write_physical_body_data(
    dispatcher: Arc<Dispatcher>,
    paths: &Paths,
    name: String,
    root: &Node,
    bin: Option<&[u8]>,
) -> Result<()> {
    let colliders_root = get_colliders_root(&root);
    let mut colliders = Vec::new();

    if let Some(colliders_root) = colliders_root {
        for collider in colliders_root.children() {
            let Some(collider_extras) = extract_collider_extras(&collider) else {
                continue;
            };

            let (translation, rotation, scale) = collider.transform().decomposed();

            let collider_shape = match collider_extras.collider_shape {
                ColliderShapeType::Box => ColliderShape::Box {
                    size: scale,
                },
                ColliderShapeType::ConvexHull => {
                    let Some(collider_mesh) = collider.mesh() else {
                        error!("Failed to extract collider ConvexHull");

                        continue;
                    };

                    let mut vertices = Vec::new();

                    for primitive in collider_mesh.primitives() {
                        let reader = primitive.reader(|buffer| match buffer.source() {
                            buffer::Source::Bin => None,
                            buffer::Source::Uri(_) => bin,
                        });

                        let primitive_vertices = reader.read_positions();

                        if let Some(primitive_vertices) = primitive_vertices {
                            vertices.extend(primitive_vertices);
                        }
                    }

                    ColliderShape::ConvexHull { vertices }
                }
            };

            let body_collider_data = ColliderData {
                collider_name: collider_extras.collider_name,

                collider_shape,

                density: collider_extras.density,
                friction: collider_extras.friction,
                restitution: collider_extras.restitution,

                translation,
                rotation,
            };

            colliders.push(body_collider_data);
        }
    }

    let path = paths.target.join(&name).with_extension("PHYSICAL_BODY");
    let physical_body_data = PhysicalBodyData { 
        name, 
        colliders,
    };
   
    let physical_body_data_bytes = to_bytes::<Error>(&physical_body_data)?.into_vec();

    dispatcher.dispatch(BuildTask::WriteFile(WriteFileTask {
        target_path: path,

        data: physical_body_data_bytes,
    }));
    
    Ok(())
}

fn get_colliders_root<'a>(mesh_root: &Node<'a>) -> Option<Node<'a>> {
    mesh_root.children().find(|node| {
        node.name()
            .map(|name| name.ends_with(".Colliders"))
            .unwrap_or(false)
    })
}
