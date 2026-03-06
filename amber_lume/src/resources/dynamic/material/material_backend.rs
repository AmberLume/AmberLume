use crate::resources::index::resource_index::ResourceIndex;
use anyhow::Result;
use rkyv::{access, deserialize};
use std::sync::Arc;
use rkyv::rancor::Error;
use tracing::info;
use builder::data::material_data::{ArchivedMaterialData, MaterialData};
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::buffer::typed::material_buffer::MaterialGpuData;
use crate::render::vulkan::resource_loader::ResourceLoader;
use crate::resources::dynamic::image::image_backend::ImageBackend;
use crate::resources::dynamic::image::image_config::ImageConfig;
use crate::resources::dynamic::material::material_config::MaterialConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::dynamic::resource_provider::{ResourceId, ResourceProvider};
use crate::resources::persistent::persistent_resources::PersistentResources;

pub struct MaterialBackend {
    buffer_manager: Arc<BufferManager>,
    image_provider: Arc<ResourceProvider<ImageBackend>>,
    resource_index: Arc<ResourceIndex>,

    resource_loader: Arc<ResourceLoader>,

    default_material: MaterialGpuData,
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
        let default_material = MaterialGpuData::create(
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

    fn upload_material(&self, resource_id: ResourceId, material_gpu_data: MaterialGpuData) -> Result<()> {
        self.resource_loader.load_buffer_at(
            &self.buffer_manager.material_buffer.at(resource_id),
            &[material_gpu_data],
        )?;

        info!("Uploaded material: index: {}, data: {:?}", resource_id, material_gpu_data);

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
        let material_file_name = format!("{}.{}", config.name, "material");
        let material_bytes = self.resource_index.get_resource(&material_file_name)?;

        let archived = access::<ArchivedMaterialData, Error>(&material_bytes)?;

        let material_data = deserialize::<MaterialData, Error>(archived)?;

        let mut images = Vec::new();

        let color_texture_id = if let Some(base_texture_id) = material_data.base_texture_id {
            let image = self.image_provider.get_or_load(ImageConfig {
                name: base_texture_id,
            });

            images.push(image.clone());

            image.id
        } else {
            self.default_material.color_texture_index
        };

        let normal_texture_id = if let Some(normal_texture_id) = material_data.normal_texture_id {
            let image = self.image_provider.get_or_load(ImageConfig {
                name: normal_texture_id,
            });

            images.push(image.clone());

            image.id
        } else {
            self.default_material.normal_texture_index
        };

        let occlusion_roughness_metallic_texture_id = if let Some(occlusion_roughness_metallic_texture_id) = material_data.occlusion_roughness_metalic_texture_id {
            let image = self.image_provider.get_or_load(ImageConfig {
                name: occlusion_roughness_metallic_texture_id,
            });

            images.push(image.clone());

            image.id
        } else {
            self.default_material.occlusion_roughness_metallic_texture_index
        };

        let material_data = MaterialGpuData::create(
            material_data.base_color_factor,
            material_data.roughness_factor,
            material_data.metallic_factor,
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
