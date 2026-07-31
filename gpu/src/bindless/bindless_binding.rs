use std::sync::Arc;
use ash::vk::ImageView;
use crate::bindless::bindless_image::BindlessImage;
use crate::binding_layout::managed_descriptor_set::ManagedDescriptorSet;
use crate::bindless::bindless_image_array::BindlessImageArray;
use crate::index::index_manager::IndexManager;

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
        let resource_id = self.index_manager.acquire()?;

        self.descriptor_set.write(resource_id, image_view);

        Some(BindlessImage::new(resource_id, self.index_manager.clone()))
    }

    pub fn acquire_image_array(&self, image_views: &[ImageView]) -> Option<BindlessImageArray> {
        let mut resource_ids = Vec::with_capacity(image_views.len());

        for image_view in image_views {
            let Some(slot) = self.index_manager.acquire() else {
                for slot in resource_ids {
                    self.index_manager.release(slot);
                }
                return None;
            };

            self.descriptor_set.write(slot, *image_view);
            resource_ids.push(slot);
        }

        Some(BindlessImageArray::new(resource_ids, self.index_manager.clone()))
    }

    pub fn update(&self) {
        self.index_manager.update();
    }
}
