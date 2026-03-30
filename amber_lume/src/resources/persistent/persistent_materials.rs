use std::sync::Arc;
use anyhow::Result;
use crate::resources::dynamic::material::material_backend::MaterialBackend;
use crate::resources::dynamic::material::material_config::MaterialConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::persistent::persistent_images::PersistentImages;

pub struct PersistentMaterials {
    pub default: Arc<ResRef>,
}

impl PersistentMaterials {
    pub fn create(
        material_provider: &ResourceProvider<MaterialBackend>,
        persistent_images: &PersistentImages,
    ) -> Result<Self> {
        let default = material_provider.acquire_sync(MaterialConfig::InBuilt {
            base_color_factor: [1.0, 0.0, 1.0, 1.0],
            roughness_factor: 1.0,
            metallic_factor: 1.0,
            color_image: persistent_images.white_pixel.clone(),
            normal_image: persistent_images.neutral_normal.clone(),
            orm_image: persistent_images.neutral_orm.clone(),
        });

        Ok(Self {
            default,
        })
    }

    pub fn destroy(self) {
        drop(self.default);
    }
}
