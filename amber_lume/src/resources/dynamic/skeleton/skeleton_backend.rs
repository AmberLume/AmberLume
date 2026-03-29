use crate::resources::alpaca_resource_reader::alpaca_resource_reader::AlpacaResourceReader;
use anyhow::Result;
use rkyv::access;
use std::sync::Arc;
use rkyv::rancor::Error;
use tracing::info;
use builder::data::skeleton_data::ArchivedSkeletonData;
use crate::ids::SliceIndex;
use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::buffer::typed::skeleton::skeleton_bones_buffer::SkeletonBoneGPU;
use crate::render::buffer::typed::skeleton::skeleton_buffer::SkeletonGPU;
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_managers::IndexManagers;
use crate::resources::dynamic::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::dynamic::resource_provider::ResourceId;
use crate::resources::dynamic::skeleton::skeleton_config::SkeletonConfig;
use crate::resources::persistent::persistent_resources::PersistentResources;

pub struct SkeletonBackend {
    buffer_manager: Arc<BufferManager>,
    alpaca_resource_reader: Arc<AlpacaResourceReader>,
    index_managers: Arc<IndexManagers>,

    persistent_resources: Arc<PersistentResources>,

    resource_loader: Arc<ResourceLoader>,
}

impl SkeletonBackend {
    pub fn new(
        buffer_manager: Arc<BufferManager>,
        persistent_resources: Arc<PersistentResources>,
        index_managers: Arc<IndexManagers>,
        alpaca_resource_reader: Arc<AlpacaResourceReader>,
        resource_loader: Arc<ResourceLoader>,
    ) -> Self {
        Self {
            buffer_manager,
            alpaca_resource_reader,
            index_managers,

            persistent_resources,

            resource_loader,
        }
    }

    fn upload_skeleton(&self, resource_id: ResourceId, data: SkeletonGPU) -> Result<()> {
        self.resource_loader.load_buffer_at(
            &self.buffer_manager.skeletons_buffer.slice_at(SliceIndex { value: resource_id }),
            &[data],
        )?;

        info!("Uploaded Skeleton: index: {}, data: {:?}", resource_id, data);

        Ok(())
    }

    fn upload_skeleton_bones(&self, resource_id: ResourceId, data: &[SkeletonBoneGPU]) -> Result<()> {
        self.resource_loader.load_buffer_at(
            &self.buffer_manager.skeleton_bones_buffer.slice_at(SliceIndex { value: resource_id }),
            &data,
        )?;

        info!("Uploaded SkeletonBones: index: {}, count: {:?}", resource_id, data.len());

        Ok(())
    }
}

pub struct ManagedSkeleton {
    pub name: String,

    pub bones_offset: u32,
    pub bones_count: u32,
}

impl ResourceBackend for SkeletonBackend {
    type Config = SkeletonConfig;
    type Output = ManagedSkeleton;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        let skeleton_bytes = self.alpaca_resource_reader.get_resource(&config.resource_key)?;
        let archived_skeleton_data = access::<ArchivedSkeletonData, Error>(&skeleton_bytes)?;

        let bones = archived_skeleton_data.bones.iter().map(|archived_bone| {
            SkeletonBoneGPU::create(
                archived_bone.parent_index.to_native(),
                archived_bone.inverse_bind_matrix.map(|s| s.map(|v| v.into())),
            )
        }).collect::<Vec<_>>();
        let bones_count = bones.len() as u32;

        let bones_slice_start = self.index_managers.skeleton_bones_index_manager.acquire_range(bones.len() as u32).unwrap();

        self.upload_skeleton_bones(bones_slice_start, &bones)?;
        self.upload_skeleton(*id, SkeletonGPU::create(
            bones_slice_start,
            bones_count,
        ))?;

        Ok(ManagedSkeleton {
            name: archived_skeleton_data.name.to_string(),

            bones_offset: bones_slice_start,
            bones_count,
        })
    }

    fn set_default(&self, id: &ResourceId) -> Result<()> {
        self.upload_skeleton(*id, self.persistent_resources.skeletons.identity.1)?;

        Ok(())
    }

    fn destroy_resource(&self, _resource: Self::Output) -> Result<()> {
        Ok(())
    }
}
