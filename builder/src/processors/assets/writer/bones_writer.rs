use crate::dispatcher::Dispatcher;
use anyhow::Result;
use std::sync::Arc;
use rkyv::rancor::Error;
use rkyv::to_bytes;
use crate::build_target::BuildTarget;
use crate::build_task::BuildTask;
use resource_data::skeleton_data::SkeletonData;
use crate::processors::assets::adapter::skeleton_adapter::Skeleton;
use crate::processors::utils::resource_key;

pub fn write_bones_data(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    skeleton: Skeleton,
) -> Result<SkeletonData> {
    let resource_key = resource_key(build_target, &skeleton.name, "SKELETON");

    let skeleton_data = SkeletonData {
        name: skeleton.name,

        bones: skeleton.bones,
    };

    dispatcher.dispatch(BuildTask::archive(
        build_target,
        &resource_key,
        to_bytes::<Error>(&skeleton_data)?.to_vec(),
    ));

    Ok(skeleton_data)
}
