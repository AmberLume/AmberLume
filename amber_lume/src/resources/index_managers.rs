use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use crate::limits::renderer_limits::RendererLimits;
use crate::resources::index::index_manager::IndexManager;

pub struct IndexManagers {
    pub index_index_manager: Arc<IndexManager>,
    pub vertex_index_manager: Arc<IndexManager>,

    pub submesh_index_manager: Arc<IndexManager>,

    pub texture_array_descriptors_index_manager: Arc<IndexManager>,
    pub shadow_descriptors_index_manager: Arc<IndexManager>,
    pub shadow_array_descriptors_index_manager: Arc<IndexManager>,
}

impl IndexManagers {
    pub fn create(
        renderer_limits: &RendererLimits,
        frames_in_flight: u32,
        current_frame: Arc<AtomicU64>,
    ) -> Self {
        let index_index_manager = IndexManager::new(renderer_limits.render_resource_limits.max_indices, frames_in_flight, current_frame.clone());
        let vertex_index_manager = IndexManager::new(renderer_limits.render_resource_limits.max_vertices, frames_in_flight, current_frame.clone());

        let submesh_index_manager = IndexManager::new(renderer_limits.render_resource_limits.max_submeshes, frames_in_flight, current_frame.clone());

        let texture_array_descriptors_index_manager = IndexManager::new(renderer_limits.image_resource_limits.max_texture_array_descriptors, frames_in_flight, current_frame.clone());
        let shadow_descriptors_index_manager = IndexManager::new(renderer_limits.image_resource_limits.max_shadow_descriptors, frames_in_flight, current_frame.clone());
        let shadow_array_descriptors_index_manager = IndexManager::new(renderer_limits.image_resource_limits.max_shadow_array_descriptors, frames_in_flight, current_frame.clone());

        Self {
            index_index_manager: Arc::new(index_index_manager),
            vertex_index_manager: Arc::new(vertex_index_manager),

            submesh_index_manager: Arc::new(submesh_index_manager),

            texture_array_descriptors_index_manager: Arc::new(texture_array_descriptors_index_manager),
            shadow_descriptors_index_manager: Arc::new(shadow_descriptors_index_manager),
            shadow_array_descriptors_index_manager: Arc::new(shadow_array_descriptors_index_manager),
        }
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
