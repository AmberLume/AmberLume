use gpu_data::SkeletonBoneGPU;
use std::sync::Arc;
use anyhow::Result;
use glam::Mat4;
use index_allocator::ResourceLimits;
use resource_residency::ResRef;
use resource_residency::ResourceProvider;

use crate::store::providers::skeleton::skeleton_backend::SkeletonBackend;
use crate::store::providers::skeleton::skeleton_config::SkeletonConfig;

pub struct PersistentSkeletons {
    pub identity: Arc<ResRef>,
}

impl PersistentSkeletons {
    pub fn create(
        skeleton_provider: &ResourceProvider<SkeletonBackend>,
        limits: &ResourceLimits,
    ) -> Result<Self> {
        let identity_bone = SkeletonBoneGPU::create(-1, Mat4::IDENTITY.to_cols_array_2d());

        let identity = skeleton_provider.acquire_sync(SkeletonConfig::InBuilt {
            name: String::from("identity"),
            bones: vec![identity_bone; limits.max_bones_per_skeleton as usize],
        });

        Ok(Self {
             identity,
        })
    }
    
    pub fn destroy(self) {
        drop(self.identity);
    }
}
