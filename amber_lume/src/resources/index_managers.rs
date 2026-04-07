use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use crate::limits::ResourceLimits;
use crate::resources::index::index_manager::IndexManager;

pub struct IndexManagers {
    pub texture_array_descriptors_index_manager: Arc<IndexManager>,
    pub shadow_descriptors_index_manager: Arc<IndexManager>,
    pub shadow_array_descriptors_index_manager: Arc<IndexManager>,
}

impl IndexManagers {
    pub fn create(
        limits: &ResourceLimits,
        frames_in_flight: u32,
        current_frame: Arc<AtomicU64>,
    ) -> Self {
        let texture_array_descriptors_index_manager = IndexManager::new(limits.max_texture_array_descriptors, frames_in_flight, current_frame.clone());
        let shadow_descriptors_index_manager = IndexManager::new(limits.max_shadow_descriptors, frames_in_flight, current_frame.clone());
        let shadow_array_descriptors_index_manager = IndexManager::new(limits.max_shadow_array_descriptors, frames_in_flight, current_frame.clone());

        Self {
            texture_array_descriptors_index_manager: Arc::new(texture_array_descriptors_index_manager),
            shadow_descriptors_index_manager: Arc::new(shadow_descriptors_index_manager),
            shadow_array_descriptors_index_manager: Arc::new(shadow_array_descriptors_index_manager),
        }
    }

    pub fn update(&self) {
        self.texture_array_descriptors_index_manager.update();
        self.shadow_descriptors_index_manager.update();
        self.shadow_array_descriptors_index_manager.update();
    }
}
