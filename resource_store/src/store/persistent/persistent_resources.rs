use crate::store::persistent::persistent_images::PersistentImages;
use crate::store::persistent::persistent_materials::PersistentMaterials;
use resource_residency::ResRef;
use std::sync::Arc;

pub struct PersistentResources {
    images: PersistentImages,
    materials: PersistentMaterials,
}

impl PersistentResources {
    pub fn white_pixel(&self) -> Arc<ResRef> {
        self.images.white_pixel.clone()
    }

    pub fn default_material(&self) -> Arc<ResRef> {
        self.materials.default.clone()
    }

    pub(crate) fn create(
        images: PersistentImages,
        materials: PersistentMaterials,
    ) -> Self {
        
        Self {
            images,
            materials,
        }
    }
}
