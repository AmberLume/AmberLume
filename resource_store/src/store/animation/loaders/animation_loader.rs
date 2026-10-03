use crate::store::animation::backend::animation_backend::AnimationBackend;
use crate::store::animation::backend::animation_config::AnimationConfig;
use crate::store::skeleton::loaders::skeleton_loader::SkeletonLoader;
use anyhow::Result;
use gpu_data::AnimationFrameGPU;
use resource_data::animation_data::ArchivedAnimationData;
use resource_reader::ResourceReader;
use resource_residency::ResRef;
use resource_residency::ResourceProvider;
use rkyv::access;
use rkyv::rancor::Error;
use std::sync::Arc;

pub struct AnimationLoader {
    resource_reader: Arc<dyn ResourceReader>,

    animation_provider: Arc<ResourceProvider<AnimationBackend>>,
    skeleton_loader: Arc<SkeletonLoader>,
}

impl AnimationLoader {
    pub fn new(
        resource_reader: Arc<dyn ResourceReader>,
        animation_provider: Arc<ResourceProvider<AnimationBackend>>,
        skeleton_loader: Arc<SkeletonLoader>,
    ) -> Self {
        Self {
            resource_reader,

            animation_provider,
            skeleton_loader,
        }
    }

    pub fn load(self: &Arc<Self>, resource_key: &str) -> Result<Arc<ResRef>> {
        let loader = self.clone();
        let key = resource_key.to_string();

        self.animation_provider.get_or_load(&resource_key, move || loader.read(&key))
    }

    fn read(&self, resource_key: &str) -> Result<AnimationConfig> {
        let animation_bytes = self.resource_reader.get_resource(resource_key)?;
        let archived_animation_data = access::<ArchivedAnimationData, Error>(&animation_bytes)?;

        let skeleton = self.skeleton_loader.load(&archived_animation_data.skeleton.value)?;

        let frames = archived_animation_data.keyframes
            .iter()
            .map(|keyframe| {
                AnimationFrameGPU::create(
                    keyframe.translation.map(|v| v.into()),
                    keyframe.rotation.map(|v| v.into()),
                    keyframe.scale.map(|v| v.into())
                )
            })
            .collect::<Vec<_>>();

        Ok(AnimationConfig {
            name: archived_animation_data.name.to_string(),
            skeleton,

            duration: archived_animation_data.duration.into(),
            fps: archived_animation_data.fps.into(),
            bone_count: archived_animation_data.bone_count.into(),
            frame_count: archived_animation_data.frame_count.into(),

            frames,
        })
    }
}
