use std::sync::Arc;
use anyhow::Result;
use crate::data::alpha_mode::AlphaMode;
use crate::resources::store::persistent::persistent_images::PersistentImages;
use crate::resources::store::providers::material::material_backend::MaterialBackend;
use crate::resources::store::providers::material::material_config::MaterialConfig;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;

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
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: AlphaMode::DEFAULT_CUTOFF,
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
