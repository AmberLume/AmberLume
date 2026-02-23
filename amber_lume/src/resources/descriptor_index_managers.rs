use std::sync::Arc;
use crate::resources::descriptor_index_manager::DescriptorIndexManager;

pub struct DescriptorIndexManagers {
    pub index_index_manager: Arc<DescriptorIndexManager>,
    pub vertex_index_manager: Arc<DescriptorIndexManager>,
    pub submesh_index_manager: Arc<DescriptorIndexManager>,
    pub image_index_manager: Arc<DescriptorIndexManager>,
    pub material_index_manager: Arc<DescriptorIndexManager>,
    pub model_index_manager: Arc<DescriptorIndexManager>,
    pub pipeline_index_manager: Arc<DescriptorIndexManager>,
    pub compute_pipeline_index_manager: Arc<DescriptorIndexManager>,
}

impl DescriptorIndexManagers {
    pub fn create(
        frames_in_flight: u64,
    ) -> Self {
        let index_index_manager = DescriptorIndexManager::new(512 * 1024, frames_in_flight);
        let vertex_index_manager = DescriptorIndexManager::new(1536 * 1024, frames_in_flight);
        let submesh_index_manager = DescriptorIndexManager::new(4096, frames_in_flight);
        let image_index_manager = DescriptorIndexManager::new(4096, frames_in_flight);
        let material_index_manager = DescriptorIndexManager::new(4096, frames_in_flight);
        let model_index_manager = DescriptorIndexManager::new(4096, frames_in_flight);
        let pipeline_index_manager = DescriptorIndexManager::new(128, frames_in_flight);
        let compute_pipeline_index_manager = DescriptorIndexManager::new(128, frames_in_flight);

        Self {
            index_index_manager: Arc::new(index_index_manager),
            vertex_index_manager: Arc::new(vertex_index_manager),
            submesh_index_manager: Arc::new(submesh_index_manager),
            image_index_manager: Arc::new(image_index_manager),
            material_index_manager: Arc::new(material_index_manager),
            model_index_manager: Arc::new(model_index_manager),
            pipeline_index_manager: Arc::new(pipeline_index_manager),
            compute_pipeline_index_manager: Arc::new(compute_pipeline_index_manager),
        }
    }
}
