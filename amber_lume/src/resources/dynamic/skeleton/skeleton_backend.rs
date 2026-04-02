use crate::resources::alpaca_resource_reader::alpaca_resource_reader::AlpacaResourceReader;
use anyhow::Result;
use rkyv::access;
use std::sync::Arc;
use rkyv::rancor::Error;
use tracing::info;
use builder::data::skeleton_data::ArchivedSkeletonData;
use crate::ids::SliceIndex;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::buffer::typed::skeleton::skeleton_bones_buffer::SkeletonBoneGPU;
use crate::render::buffer::typed::skeleton::skeleton_buffer::SkeletonGPU;
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::dynamic::resource_backend::ResourceBackend;
use crate::resources::dynamic::resource_provider::ResourceId;
use crate::resources::dynamic::skeleton::skeleton_backend_statistics::SkeletonBackendStatistics;
use crate::resources::dynamic::skeleton::skeleton_config::SkeletonConfig;
use crate::resources::range_allocator::range_allocator::{Allocation, RangeAllocator};

pub struct SkeletonBackend {
    buffer_manager: Arc<BufferManager>,
    alpaca_resource_reader: Arc<AlpacaResourceReader>,

    bone_allocator: RangeAllocator,

    resource_loader: Arc<ResourceLoader>,
}

impl SkeletonBackend {
    pub fn new(
        renderer_limits: &RendererLimits,
        buffer_manager: Arc<BufferManager>,
        alpaca_resource_reader: Arc<AlpacaResourceReader>,
        resource_loader: Arc<ResourceLoader>,
    ) -> Self {
        let bone_allocator = RangeAllocator::new(
            renderer_limits.render_resource_limits.max_skeleton_bones,
        );

        Self {
            buffer_manager,
            alpaca_resource_reader,

            bone_allocator,

            resource_loader,
        }
    }

    fn upload_skeleton(&self, resource_id: ResourceId, data: SkeletonGPU) -> Result<()> {
        self.resource_loader.load_buffer_at(
            &self.buffer_manager.skeletons_buffer.slice_at(SliceIndex::from(resource_id)),
            &[data],
        )?;

        info!("Uploaded Skeleton: index: {}, data: {:?}", resource_id, data);

        Ok(())
    }

    fn upload_skeleton_bones(&self, resource_id: ResourceId, data: &[SkeletonBoneGPU]) -> Result<()> {
        self.resource_loader.load_buffer_at(
            &self.buffer_manager.skeleton_bones_buffer.slice_at(SliceIndex::from(resource_id)),
            &data,
        )?;

        info!("Uploaded SkeletonBones: index: {}, count: {:?}", resource_id, data.len());

        Ok(())
    }
}

pub struct ManagedSkeleton {
    pub name: String,

    pub bones_allocation: Allocation,
}

impl ResourceBackend for SkeletonBackend {
    type Config = SkeletonConfig;
    type Output = ManagedSkeleton;
    type Statistics = SkeletonBackendStatistics;

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        match config {
            SkeletonConfig::Alpaca { resource_key } => {
                let skeleton_bytes = self.alpaca_resource_reader.get_resource(&resource_key)?;
                let archived_skeleton_data = access::<ArchivedSkeletonData, Error>(&skeleton_bytes)?;

                let name = archived_skeleton_data.name.to_string();

                let bones = archived_skeleton_data.bones.iter().map(|archived_bone| {
                    SkeletonBoneGPU::create(
                        archived_bone.parent_index.to_native(),
                        archived_bone.inverse_bind_matrix.map(|s| s.map(|v| v.into())),
                    )
                }).collect::<Vec<_>>();

                let bones_allocation = self.bone_allocator.allocate(bones.len() as u32).unwrap();

                self.upload_skeleton_bones(bones_allocation.offset, &bones)?;

                self.upload_skeleton(*id, SkeletonGPU::create(
                    bones_allocation.offset,
                    bones_allocation.size,
                ))?;

                Ok(ManagedSkeleton {
                    name,

                    bones_allocation,
                })
            }
            SkeletonConfig::InBuilt {
                name,
                bones,
            } => {
                let bones_allocation = self.bone_allocator.allocate(bones.len() as u32).unwrap();

                self.upload_skeleton_bones(bones_allocation.offset, &bones)?;

                self.upload_skeleton(*id, SkeletonGPU::create(
                    bones_allocation.offset,
                    bones_allocation.size,
                ))?;

                Ok(ManagedSkeleton {
                    name,

                    bones_allocation,
                })
            }
        }
    }

    fn erase(&self, _id: &ResourceId) -> Result<()> {
        // self.upload_skeleton(*id, self.default_skeleton.)?;

        Ok(())
    }

    fn statistics(&self) -> Self::Statistics {
        Self::Statistics {
            bone: self.bone_allocator.statistics(),
        }
    }
    
    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        self.bone_allocator.release(resource.bones_allocation);

        info!(
            "Destroyed skeleton: {}, allocation [{}..+{}]",
            resource.name,
            resource.bones_allocation.offset,
            resource.bones_allocation.size,
        );

        Ok(())
    }
}
