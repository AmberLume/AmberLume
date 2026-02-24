use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use crate::resources::descriptor_index_manager::IndexManager;

pub struct IndexManagers {
    pub index_index_manager: Arc<IndexManager>,
    pub vertex_index_manager: Arc<IndexManager>,
    pub submesh_index_manager: Arc<IndexManager>,
    pub image_descriptors_index_manager: Arc<IndexManager>,
    pub shadow_descriptors_index_manager: Arc<IndexManager>,
    pub material_index_manager: Arc<IndexManager>,
    pub model_index_manager: Arc<IndexManager>,
    pub pipeline_index_manager: Arc<IndexManager>,
    pub compute_pipeline_index_manager: Arc<IndexManager>,
}

impl IndexManagers {
    pub fn create(
        frames_in_flight: u64,
        current_frame: Arc<AtomicU64>,
    ) -> Self {
        let index_index_manager = IndexManager::new(512 * 1024, frames_in_flight, current_frame.clone());
        let vertex_index_manager = IndexManager::new(1536 * 1024, frames_in_flight, current_frame.clone());
        let submesh_index_manager = IndexManager::new(4096, frames_in_flight, current_frame.clone());
        let image_descriptors_index_manager = IndexManager::new(2048, frames_in_flight, current_frame.clone());
        let shadow_descriptors_index_manager = IndexManager::new(2048, frames_in_flight, current_frame.clone());
        let material_index_manager = IndexManager::new(4096, frames_in_flight, current_frame.clone());
        let model_index_manager = IndexManager::new(4096, frames_in_flight, current_frame.clone());
        let pipeline_index_manager = IndexManager::new(128, frames_in_flight, current_frame.clone());
        let compute_pipeline_index_manager = IndexManager::new(128, frames_in_flight, current_frame.clone());

        Self {
            index_index_manager: Arc::new(index_index_manager),
            vertex_index_manager: Arc::new(vertex_index_manager),
            submesh_index_manager: Arc::new(submesh_index_manager),
            image_descriptors_index_manager: Arc::new(image_descriptors_index_manager),
            shadow_descriptors_index_manager: Arc::new(shadow_descriptors_index_manager),
            material_index_manager: Arc::new(material_index_manager),
            model_index_manager: Arc::new(model_index_manager),
            pipeline_index_manager: Arc::new(pipeline_index_manager),
            compute_pipeline_index_manager: Arc::new(compute_pipeline_index_manager),
        }
    }
}
