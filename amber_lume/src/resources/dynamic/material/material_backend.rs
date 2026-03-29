use crate::resources::index::resource_index::ResourceIndex;
use anyhow::Result;
use rkyv::access;
use std::sync::Arc;
use rkyv::rancor::Error;
use tracing::info;
use builder::data::material_data::ArchivedMaterialData;
use crate::ids::SliceIndex;
use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::buffer::typed::materials_buffer::MaterialGPU;
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::dynamic::image::image_backend::ImageBackend;
use crate::resources::dynamic::image::image_config::ImageConfig;
use crate::resources::dynamic::material::material_config::MaterialConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::dynamic::resource_provider::{ResourceId, ResourceProvider};
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::resources::utils::slice_utils::as_f32_slice;

pub struct MaterialBackend {
    buffer_manager: Arc<BufferManager>,
    image_provider: Arc<ResourceProvider<ImageBackend>>,
    resource_index: Arc<ResourceIndex>,

    resource_loader: Arc<ResourceLoader>,

    default_material: MaterialGPU,
}

pub struct ManagedMaterial {
    pub images: Vec<Arc<ResRef>>,
}

impl MaterialBackend {
    pub fn new(
        buffer_manager: Arc<BufferManager>,
        image_provider: Arc<ResourceProvider<ImageBackend>>,
        resource_index: Arc<ResourceIndex>,
        resource_loader: Arc<ResourceLoader>,
        persistent_resources: &PersistentResources,
    ) -> Self {
        let default_material = MaterialGPU::create(
            [0.7, 0.2, 0.7, 1.0],
            1.0,
            1.0,
            persistent_resources.images.white_pixel.descriptor_index,
            persistent_resources.images.default_normal.descriptor_index,
            persistent_resources.images.default_occlusion_roughness_metallic.descriptor_index,
        );

        Self {
            buffer_manager,
            image_provider,
            resource_index,

            resource_loader,

            default_material,
        }
    }

    fn upload_material(&self, resource_id: ResourceId, material_gpu: MaterialGPU) -> Result<()> {
        self.resource_loader.load_buffer_at(
            &self.buffer_manager.material_buffer.slice_at(SliceIndex { value: resource_id }),
            &[material_gpu],
        )?;

        info!("Uploaded material: index: {}, data: {:?}", resource_id, material_gpu);

        Ok(())
    }
}


impl ResourceBackend for MaterialBackend {
    type Config = MaterialConfig;
    type Output = ManagedMaterial;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        let material_bytes = self.resource_index.get_resource(&config.resource_key)?;

        let archived_material_data = access::<ArchivedMaterialData, Error>(&material_bytes)?;

        let mut images = Vec::new();

        let color_texture_id = if let Some(base_texture_id) = archived_material_data.base_texture_id.as_ref() {
            let image = self.image_provider.get_or_load(ImageConfig {
                resource_key: base_texture_id.value.to_string(),
            });

            images.push(image.clone());

            image.id
        } else {
            self.default_material.color_texture_index
        };

        let normal_texture_id = if let Some(normal_texture_id) = archived_material_data.normal_texture_id.as_ref() {
            let image = self.image_provider.get_or_load(ImageConfig {
                resource_key: normal_texture_id.value.to_string(),
            });

            images.push(image.clone());

            image.id
        } else {
            self.default_material.normal_texture_index
        };

        let occlusion_roughness_metallic_texture_id = if let Some(occlusion_roughness_metallic_texture_id) = archived_material_data.occlusion_roughness_metallic_texture_id.as_ref() {
            let image = self.image_provider.get_or_load(ImageConfig {
                resource_key: occlusion_roughness_metallic_texture_id.value.to_string(),
            });

            images.push(image.clone());

            image.id
        } else {
            self.default_material.occlusion_roughness_metallic_texture_index
        };

        let material_data = MaterialGPU::create(
            as_f32_slice(&archived_material_data.base_color_factor),
            archived_material_data.roughness_factor.into(),
            archived_material_data.metallic_factor.into(),
            color_texture_id,
            normal_texture_id,
            occlusion_roughness_metallic_texture_id,
        );

        self.upload_material(*id, material_data)?;

        Ok(ManagedMaterial {
            images,
        })
    }

    fn set_default(&self, id: &ResourceId) -> Result<()> {
        self.upload_material(*id, self.default_material)?;

        Ok(())
    }

    fn destroy_resource(&self, _resource: Self::Output) -> Result<()> {
        Ok(())
    }
}
