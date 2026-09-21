use gpu_data::MaterialGPU;
use anyhow::Result;
use rkyv::access;
use std::sync::Arc;
use rkyv::rancor::Error;
use tracing::info;
use resource_data::material_data::ArchivedMaterialData;
use index_allocator::ResourceLimits;
use gpu::ResourceTransfer;
use resource_reader::ResourceReader;
use resource_residency::ResRef;
use resource_residency::ResourceBackend;
use resource_residency::ResourceProvider;
use index_allocator::ResourceId;
use crate::store::persistent::persistent_images::PersistentImages;
use crate::store::providers::image::image_backend::ImageBackend;
use crate::store::providers::image::image_config::ImageConfig;
use gpu::SingleAllocation;
use crate::store::providers::material::material_config::MaterialConfig;

pub struct MaterialBackend {
    resource_reader: Arc<dyn ResourceReader>,
    resource_transfer: Arc<ResourceTransfer>,

    image_provider: Arc<ResourceProvider<ImageBackend>>,

    material: Arc<SingleAllocation<MaterialGPU>>,
    
    default_color_image: Arc<ResRef>,
    default_normal_image: Arc<ResRef>,
    default_orm_image: Arc<ResRef>,
}

pub struct ManagedMaterial {
    pub images: Vec<Arc<ResRef>>,
}

impl MaterialBackend {
    pub(crate) fn new(
        limits: &ResourceLimits,
        material: Arc<SingleAllocation<MaterialGPU>>,
        image_provider: Arc<ResourceProvider<ImageBackend>>,
        resource_reader: Arc<dyn ResourceReader>,
        resource_transfer: Arc<ResourceTransfer>,
        persistent_images: &PersistentImages,
    ) -> Result<Self> {
        resource_transfer.load_buffer_at(
            material.allocation.range(0, material.allocation.size),
            &vec![MaterialGPU::DEFAULT; limits.max_materials as usize],
        )?;

        Ok(Self {
            resource_reader,
            resource_transfer,

            image_provider,

            material,
            
            default_color_image: persistent_images.white_pixel.clone(),
            default_normal_image: persistent_images.neutral_normal.clone(),
            default_orm_image: persistent_images.neutral_orm.clone(),
        })
    }

    fn upload_material(&self, id: ResourceId, data: MaterialGPU) -> Result<()> {
        self.resource_transfer.load_buffer_at(
            self.material.at(id.inner),
            &[data],
        )?;

        info!("Uploaded material: index: {}, data: {:?}", id.inner, data);

        Ok(())
    }
}

impl ResourceBackend for MaterialBackend {
    type Config = MaterialConfig;
    type Output = ManagedMaterial;
    type Statistics = ();

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        match config {
            MaterialConfig::Alpaca { resource_key } => {
                let material_bytes = self.resource_reader.get_resource(&resource_key)?;
                let archived_material_data = access::<ArchivedMaterialData, Error>(&material_bytes)?;

                let color_image = if let Some(base_resource_key) = archived_material_data.base_texture_id.as_ref() {
                    self.image_provider.get_or_load(ImageConfig::Alpaca {
                        resource_key: base_resource_key.value.to_string(),
                    })?
                } else {
                    self.default_color_image.clone()
                };

                let normal_image = if let Some(normal_resource_key) = archived_material_data.normal_texture_id.as_ref() {
                    self.image_provider.get_or_load(ImageConfig::Alpaca {
                        resource_key: normal_resource_key.value.to_string(),
                    })?
                } else {
                    self.default_normal_image.clone()
                };

                let orm_image = if let Some(orm_resource_key) = archived_material_data.orm_texture_id.as_ref() {
                    self.image_provider.get_or_load(ImageConfig::Alpaca {
                        resource_key: orm_resource_key.value.to_string(),
                    })?
                } else {
                    self.default_orm_image.clone()
                };

                self.upload_material(*id, MaterialGPU::create(
                    archived_material_data.base_color_factor.map(|v| v.into()),
                    archived_material_data.roughness_factor.into(),
                    archived_material_data.metallic_factor.into(),
                    (&archived_material_data.alpha_mode).into(),
                    archived_material_data.alpha_cutoff.into(),
                    color_image.id.inner,
                    normal_image.id.inner,
                    orm_image.id.inner,
                ))?;

                Ok(ManagedMaterial {
                    images: vec![
                        color_image,
                        normal_image,
                        orm_image,
                    ],
                })
            }
            MaterialConfig::InBuilt {
                base_color_factor,
                roughness_factor,
                metallic_factor,

                alpha_mode,
                alpha_cutoff,

                color_image,
                normal_image,
                orm_image,
            } => {
                self.upload_material(*id, MaterialGPU::create(
                    base_color_factor,
                    roughness_factor,
                    metallic_factor,
                    alpha_mode,
                    alpha_cutoff,
                    color_image.id.inner,
                    normal_image.id.inner,
                    orm_image.id.inner,
                ))?;

                Ok(ManagedMaterial {
                    images: vec![
                        color_image,
                        normal_image,
                        orm_image,
                    ],
                })
            }
        }
    }

    fn erase(&self, id: &ResourceId) -> Result<()> {
        self.upload_material(*id, MaterialGPU::DEFAULT)?;

        Ok(())
    }

    fn statistics(&self) -> Self::Statistics {
        ()
    }

    fn destroy_resource(&self, _resource: Self::Output) -> Result<()> {
        Ok(())
    }
}
