use std::sync::Arc;
use ash::vk::ImageView;
use crate::resources::bindless::bindless_image::BindlessImage;
use crate::resources::binding_layout::managed_descriptor_set::ManagedDescriptorSet;
use crate::resources::bindless::bindless_image_array::BindlessImageArray;
use crate::resources::index::index_manager::IndexManager;

pub struct BindlessBinding {
    descriptor_set: ManagedDescriptorSet,
    index_manager: Arc<IndexManager>,
}

impl BindlessBinding {
    pub fn new(
        descriptor_set: ManagedDescriptorSet,
        index_manager: IndexManager,
    ) -> Self {
        Self {
            descriptor_set,
            index_manager: Arc::new(index_manager),
        }
    }

    pub fn acquire_image(&self, image_view: ImageView) -> Option<BindlessImage> {
        let slot = self.index_manager.acquire()?;

        self.descriptor_set.write(slot, image_view);

        Some(BindlessImage::new(slot, self.index_manager.clone()))
    }

    pub fn acquire_image_array(&self, image_views: &[ImageView]) -> Option<BindlessImageArray> {
        let mut slots = Vec::with_capacity(image_views.len());

        for image_view in image_views {
            let Some(slot) = self.index_manager.acquire() else {
                for slot in slots {
                    self.index_manager.release(slot);
                }
                return None;
            };

            self.descriptor_set.write(slot, *image_view);
            slots.push(slot);
        }

        Some(BindlessImageArray::new(slots, self.index_manager.clone()))
    }

    pub fn update(&self) {
        self.index_manager.update();
    }
}
