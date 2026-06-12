use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use crate::limits::ResourceLimits;
use crate::resources::bindless::bindless_binding::BindlessBinding;
use crate::resources::binding_layout::descriptor_set_manager::DescriptorSetManager;
use crate::resources::index::index_manager::IndexManager;

pub struct Bindless {
    pub shadow_arrays: BindlessBinding,
    pub storage_images: BindlessBinding,
}

impl Bindless {
    pub fn new(
        descriptor_set_manager: &DescriptorSetManager,
        limits: &ResourceLimits,
        frames_in_flight: u32,
        current_frame: Arc<AtomicU64>,
    ) -> Self {
        let shadow_arrays = BindlessBinding::new(
            descriptor_set_manager.shadow_arrays_descriptor_set.clone(),
            IndexManager::new(limits.max_shadow_array_descriptors, frames_in_flight, current_frame.clone()),
        );
        let storage_images = BindlessBinding::new(
            descriptor_set_manager.storage_images_descriptor_set.clone(),
            IndexManager::new(limits.max_storage_image_descriptors, frames_in_flight, current_frame.clone()),
        );

        Self {
            shadow_arrays,
            storage_images,
        }
    }

    pub fn update(&self) {
        self.shadow_arrays.update();
        self.storage_images.update();
    }
}
