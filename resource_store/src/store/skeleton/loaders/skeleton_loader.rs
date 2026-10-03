use crate::store::skeleton::backend::skeleton_backend::SkeletonBackend;
use crate::store::skeleton::backend::skeleton_config::SkeletonConfig;
use anyhow::Result;
use gpu_data::SkeletonBoneGPU;
use resource_data::skeleton_data::ArchivedSkeletonData;
use resource_reader::ResourceReader;
use resource_residency::ResRef;
use resource_residency::ResourceProvider;
use rkyv::access;
use rkyv::rancor::Error;
use std::sync::Arc;

pub struct SkeletonLoader {
    resource_reader: Arc<dyn ResourceReader>,

    skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
}

impl SkeletonLoader {
    pub fn new(
        resource_reader: Arc<dyn ResourceReader>,
        skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
    ) -> Self {
        Self {
            resource_reader,

            skeleton_provider,
        }
    }

    pub fn load(self: &Arc<Self>, resource_key: &str) -> Result<Arc<ResRef>> {
        let loader = self.clone();
        let key = resource_key.to_string();

        self.skeleton_provider.get_or_load(&resource_key, move || loader.read(&key))
    }

    fn read(&self, resource_key: &str) -> Result<SkeletonConfig> {
        let skeleton_bytes = self.resource_reader.get_resource(resource_key)?;
        let archived_skeleton_data = access::<ArchivedSkeletonData, Error>(&skeleton_bytes)?;

        let bones = archived_skeleton_data
            .bones
            .iter()
            .map(|archived_bone| SkeletonBoneGPU::create(archived_bone.parent_index.to_native()))
            .collect::<Vec<_>>();

        Ok(SkeletonConfig {
            name: archived_skeleton_data.name.to_string(),

            bones,
        })
    }
}
