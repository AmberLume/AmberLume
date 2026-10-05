use crate::store::image::loaders::image_loader::ImageLoader;
use crate::store::material::backend::material_backend::MaterialBackend;
use crate::store::material::backend::material_config::MaterialConfig;
use crate::store::persistent::persistent_images::PersistentImages;
use anyhow::Result;
use resource_data::material_data::ArchivedMaterialData;
use resource_reader::ResourceReader;
use resource_residency::ResRef;
use resource_residency::ResourceProvider;
use rkyv::access;
use rkyv::rancor::Error;
use std::sync::Arc;

pub struct MaterialLoader {
    resource_reader: Arc<dyn ResourceReader>,

    material_provider: Arc<ResourceProvider<MaterialBackend>>,

    image_loader: Arc<ImageLoader>,

    default_color_image: Arc<ResRef>,
    default_normal_image: Arc<ResRef>,
    default_orm_image: Arc<ResRef>,
}

impl MaterialLoader {
    pub fn new(
        resource_reader: Arc<dyn ResourceReader>,
        material_provider: Arc<ResourceProvider<MaterialBackend>>,
        image_loader: Arc<ImageLoader>,
        persistent_images: &PersistentImages,
    ) -> Self {
        Self {
            resource_reader,

            material_provider,

            image_loader,

            default_color_image: persistent_images.white_pixel.clone(),
            default_normal_image: persistent_images.neutral_normal.clone(),
            default_orm_image: persistent_images.neutral_orm.clone(),
        }
    }

    pub fn load(self: &Arc<Self>, resource_key: &str) -> Result<Arc<ResRef>> {
        let loader = self.clone();
        let key = resource_key.to_string();

        self.material_provider.get_or_load(&resource_key, move || loader.read(&key))
    }

    fn read(&self, resource_key: &str) -> Result<MaterialConfig> {
        let material_bytes = self.resource_reader.get_resource(resource_key)?;
        let archived_material_data = access::<ArchivedMaterialData, Error>(&material_bytes)?;

        let color_image = if let Some(base_resource_key) = archived_material_data.base_texture_id.as_ref() {
            self.image_loader.load(&base_resource_key.value)?
        } else {
            self.default_color_image.clone()
        };

        let normal_image = if let Some(normal_resource_key) = archived_material_data.normal_texture_id.as_ref() {
            self.image_loader.load(&normal_resource_key.value)?
        } else {
            self.default_normal_image.clone()
        };

        let orm_image = if let Some(orm_resource_key) = archived_material_data.orm_texture_id.as_ref() {
            self.image_loader.load(&orm_resource_key.value)?
        } else {
            self.default_orm_image.clone()
        };

        Ok(MaterialConfig {
            base_color_factor: archived_material_data.base_color_factor.map(|v| v.into()),
            roughness_factor: archived_material_data.roughness_factor.into(),
            metallic_factor: archived_material_data.metallic_factor.into(),

            alpha_mode: (&archived_material_data.alpha_mode).into(),
            alpha_cutoff: archived_material_data.alpha_cutoff.into(),

            color_image,
            normal_image,
            orm_image,
        })
    }
}
