use crate::build_task::BuildTask;
use resource_data::physical_body_data::{ColliderData, PhysicalBodyData};
use crate::dispatcher::Dispatcher;
use anyhow::Result;
use rkyv::rancor::Error;
use rkyv::to_bytes;
use std::sync::Arc;
use crate::build_target::BuildTarget;
use crate::processors::utils::resource_key;
use crate::processors::assets::adapter::collider_adapter::Collider;

fn collider_data_from(description: Collider) -> ColliderData {
    ColliderData {
        collider_name: description.name,

        collider_shape: description.shape,

        density: description.density,
        friction: description.friction,
        restitution: description.restitution,

        translation: description.translation,
        rotation: description.rotation,
    }
}

fn dispatch_physical_body(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    name: String,
    colliders: Vec<ColliderData>,
) -> Result<()> {
    let resource_key = resource_key(build_target, &name, "PHYSICAL_BODY");
    dispatcher.dispatch(BuildTask::archive(
        build_target,
        &resource_key,
        to_bytes::<Error>(&PhysicalBodyData {
            name: name.clone(),

            colliders,
        })?.to_vec(),
    ));

    Ok(())
}

pub fn write_physical_body_data_flat(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    descriptions: Vec<Collider>,
) -> Result<()> {
    let name = build_target.name.clone();

    let colliders = descriptions.into_iter().map(collider_data_from).collect();

    dispatch_physical_body(dispatcher, build_target, name, colliders)
}

