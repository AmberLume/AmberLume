use anyhow::Result;
use glam::Mat4;
use crate::ids::SliceIndex;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::buffer::typed::skeleton::skeleton_bones_buffer::SkeletonBoneGpuData;
use crate::render::buffer::typed::skeleton::skeleton_buffer::SkeletonGpuData;
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_manager::IndexManager;
use crate::resources::dynamic::resource_provider::ResourceId;

pub struct PersistentSkeletons {
    pub identity: (ResourceId, SkeletonGpuData),
}

impl PersistentSkeletons {
    pub fn create(
        resource_loader: &ResourceLoader,
        renderer_limits: &RendererLimits,
        skeletons_index_manager: &IndexManager,
        skeleton_bones_index_manager: &IndexManager,
        buffer_manager: &BufferManager,
    ) -> Result<Self> {
        let identity_bones_count = renderer_limits.render_resource_limits.max_bones_per_skeleton;

        let identity_skeleton_resource_id = skeletons_index_manager.acquire().unwrap();
        let identity_skeleton_bones_start = skeleton_bones_index_manager.acquire_range(identity_bones_count).unwrap();

        let identity_skeleton = SkeletonGpuData::create(identity_skeleton_bones_start, identity_bones_count);
        let identity_skeleton_bone = SkeletonBoneGpuData::create(-1, Mat4::IDENTITY.to_cols_array_2d());

        resource_loader.load_buffer_at(
            &buffer_manager.skeletons_buffer.slice_at(SliceIndex { value: identity_skeleton_resource_id }),
            &[identity_skeleton],
        )?;
        resource_loader.load_buffer_at(
            &buffer_manager.skeleton_bones_buffer.slice_at(SliceIndex { value: identity_skeleton_bones_start }),
            &vec![identity_skeleton_bone; identity_bones_count as usize],
        )?;

        Ok(Self {
             identity: (identity_skeleton_resource_id, identity_skeleton),
        })
    }
}
