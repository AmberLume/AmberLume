use std::sync::Arc;
use ash::Device;
use crate::utils::debug_utils::DebugUtils;
use anyhow::Result;
use crate::factories::query_pool::query_pool::ManagedQueryPool;

pub struct QueryPoolFactory {
    device: Device,
    debug_utils: Arc<DebugUtils>,
}

impl QueryPoolFactory {
    pub fn create(
        device: Device,
        debug_utils: Arc<DebugUtils>,
    ) -> Self {
        Self {
            device,
            debug_utils,
        }
    }

    pub fn create_query_pool(
        &self,
        capacity: u32,
        label: &str,
    ) -> Result<ManagedQueryPool> {
        ManagedQueryPool::new(
            self.device.clone(),
            capacity,
            &self.debug_utils,
            &label,
        )
    }

    pub fn destroy_query_pool(&self, query_pool: ManagedQueryPool) {
        query_pool.destroy();
    }
}
