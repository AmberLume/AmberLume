use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use crate::limits::renderer_limits::RendererLimits;
use crate::resources::descriptor_index_manager::IndexManager;
use crate::resources::resource_indices_statistics::ResourceIndicesStatistics;
use crate::statistics::statistics_context::StatisticsContext;

pub struct IndexManagers {
    pub index_index_manager: Arc<IndexManager>,
    pub vertex_index_manager: Arc<IndexManager>,
    
    pub submesh_index_manager: Arc<IndexManager>,
    pub material_index_manager: Arc<IndexManager>,
    pub model_index_manager: Arc<IndexManager>,
    
    pub texture_descriptors_index_manager: Arc<IndexManager>,
    pub texture_array_descriptors_index_manager: Arc<IndexManager>,
    pub shadow_descriptors_index_manager: Arc<IndexManager>,
    pub shadow_array_descriptors_index_manager: Arc<IndexManager>,
    
    pub pipeline_index_manager: Arc<IndexManager>,
    pub compute_pipeline_index_manager: Arc<IndexManager>,
    
    statistics: Arc<ResourceIndicesStatistics>,
}

impl IndexManagers {
    pub fn create(
        renderer_limits: &RendererLimits,
        statistics_context: &StatisticsContext,
        frames_in_flight: u64,
        current_frame: Arc<AtomicU64>,
    ) -> Self {
        let index_index_manager = IndexManager::new(renderer_limits.render_resource_limits.max_indices, frames_in_flight, current_frame.clone());
        let vertex_index_manager = IndexManager::new(renderer_limits.render_resource_limits.max_vertices, frames_in_flight, current_frame.clone());
        
        let submesh_index_manager = IndexManager::new(renderer_limits.render_resource_limits.max_submeshes, frames_in_flight, current_frame.clone());
        let material_index_manager = IndexManager::new(renderer_limits.render_resource_limits.max_materials, frames_in_flight, current_frame.clone());
        let model_index_manager = IndexManager::new(renderer_limits.render_resource_limits.max_models, frames_in_flight, current_frame.clone());
        
        let texture_descriptors_index_manager = IndexManager::new(renderer_limits.image_resource_limits.max_texture_descriptors, frames_in_flight, current_frame.clone());
        let texture_array_descriptors_index_manager = IndexManager::new(renderer_limits.image_resource_limits.max_texture_array_descriptors, frames_in_flight, current_frame.clone());
        let shadow_descriptors_index_manager = IndexManager::new(renderer_limits.image_resource_limits.max_shadow_descriptors, frames_in_flight, current_frame.clone());
        let shadow_array_descriptors_index_manager = IndexManager::new(renderer_limits.image_resource_limits.max_shadow_array_descriptors, frames_in_flight, current_frame.clone());
        
        let pipeline_index_manager = IndexManager::new(128, frames_in_flight, current_frame.clone());
        let compute_pipeline_index_manager = IndexManager::new(128, frames_in_flight, current_frame.clone());

        Self {
            index_index_manager: Arc::new(index_index_manager),
            vertex_index_manager: Arc::new(vertex_index_manager),
            
            submesh_index_manager: Arc::new(submesh_index_manager),
            model_index_manager: Arc::new(model_index_manager),
            material_index_manager: Arc::new(material_index_manager),
            
            texture_descriptors_index_manager: Arc::new(texture_descriptors_index_manager),
            texture_array_descriptors_index_manager: Arc::new(texture_array_descriptors_index_manager),
            shadow_descriptors_index_manager: Arc::new(shadow_descriptors_index_manager),
            shadow_array_descriptors_index_manager: Arc::new(shadow_array_descriptors_index_manager),
            
            pipeline_index_manager: Arc::new(pipeline_index_manager),
            compute_pipeline_index_manager: Arc::new(compute_pipeline_index_manager),
            
            statistics: statistics_context.resource_indices.clone(),
        }
    }
    
    pub fn fill_statistics(&self) {
        self.statistics.fill(&self);
    }

    pub fn update(&self) {
        self.index_index_manager.update();
        self.vertex_index_manager.update();

        self.submesh_index_manager.update();

        self.texture_array_descriptors_index_manager.update();
        self.shadow_descriptors_index_manager.update();
        self.shadow_array_descriptors_index_manager.update();
    }
}
