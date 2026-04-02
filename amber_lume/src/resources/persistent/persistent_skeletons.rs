use std::sync::Arc;
use anyhow::Result;
use glam::Mat4;
use crate::limits::renderer_limits::RendererLimits;
use crate::resources::dynamic::skeleton::buffer::skeleton_bones_buffer::SkeletonBoneGPU;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::dynamic::skeleton::skeleton_backend::SkeletonBackend;
use crate::resources::dynamic::skeleton::skeleton_config::SkeletonConfig;

pub struct PersistentSkeletons {
    pub identity: Arc<ResRef>,
}

impl PersistentSkeletons {
    pub fn create(
        skeleton_provider: &ResourceProvider<SkeletonBackend>,
        renderer_limits: &RendererLimits,
    ) -> Result<Self> {
        let identity_bone = SkeletonBoneGPU::create(-1, Mat4::IDENTITY.to_cols_array_2d());

        let identity = skeleton_provider.acquire_sync(SkeletonConfig::InBuilt {
            name: String::from("identity"),
            bones: vec![identity_bone; renderer_limits.render_resource_limits.max_bones_per_skeleton as usize],
        });

        Ok(Self {
             identity,
        })
    }
    
    pub fn destroy(self) {
        drop(self.identity);
    }
}
