use crate::resources::alpaca_resource_reader::alpaca_resource_reader::AlpacaResourceReader;
use anyhow::Result;
use rkyv::access;
use std::sync::Arc;
use rkyv::rancor::Error;
use tracing::info;
use crate::data::material_data::ArchivedMaterialData;
use crate::ids::SliceIndex;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::resources::dynamic::material::buffer::materials_buffer::{create_materials_buffer, MaterialGPU};
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::dynamic::image::image_backend::ImageBackend;
use crate::resources::dynamic::image::image_config::ImageConfig;
use crate::resources::dynamic::material::material_config::MaterialConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_backend::ResourceBackend;
use crate::resources::dynamic::resource_provider::{ResourceId, ResourceProvider};
use crate::resources::persistent::persistent_images::PersistentImages;

pub struct MaterialBackend {
    resource_factories: Arc<ResourceFactories>,
    alpaca_resource_reader: Arc<AlpacaResourceReader>,
    resource_loader: Arc<ResourceLoader>,

    image_provider: Arc<ResourceProvider<ImageBackend>>,

    pub material_buffer: SliceBuffer<MaterialGPU>,
    
    default_color_image: Arc<ResRef>,
    default_normal_image: Arc<ResRef>,
    default_orm_image: Arc<ResRef>,
}

pub struct ManagedMaterial {
    pub images: Vec<Arc<ResRef>>,
}

impl MaterialBackend {
    pub fn new(
        renderer_limits: &RendererLimits,
        resource_factories: Arc<ResourceFactories>,
        image_provider: Arc<ResourceProvider<ImageBackend>>,
        alpaca_resource_reader: Arc<AlpacaResourceReader>,
        resource_loader: Arc<ResourceLoader>,
        persistent_images: &PersistentImages,
    ) -> Result<Self> {
        let material_buffer = create_materials_buffer(
            &resource_factories.buffer_factory,
            renderer_limits.render_resource_limits.max_materials,
        )?;
        
        Ok(Self {
            resource_factories,
            alpaca_resource_reader,
            resource_loader,
            
            image_provider,
            
            material_buffer,
            
            default_color_image: persistent_images.white_pixel.clone(),
            default_normal_image: persistent_images.neutral_normal.clone(),
            default_orm_image: persistent_images.neutral_orm.clone(),
        })
    }

    fn upload_material(&self, id: ResourceId, data: MaterialGPU) -> Result<()> {
        self.resource_loader.load_buffer_at(
            &self.material_buffer.slice_at(SliceIndex::from(id)),
            &[data],
        )?;

        info!("Uploaded material: index: {}, data: {:?}", id, data);

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
                let material_bytes = self.alpaca_resource_reader.get_resource(&resource_key)?;
                let archived_material_data = access::<ArchivedMaterialData, Error>(&material_bytes)?;

                let color_image = if let Some(base_resource_key) = archived_material_data.base_texture_id.as_ref() {
                    self.image_provider.get_or_load(ImageConfig::Alpaca {
                        resource_key: base_resource_key.value.to_string(),
                    })
                } else {
                    self.default_color_image.clone()
                };

                let normal_image = if let Some(normal_resource_key) = archived_material_data.normal_texture_id.as_ref() {
                    self.image_provider.get_or_load(ImageConfig::Alpaca {
                        resource_key: normal_resource_key.value.to_string(),
                    })
                } else {
                    self.default_normal_image.clone()
                };

                let orm_image = if let Some(orm_resource_key) = archived_material_data.occlusion_roughness_metallic_texture_id.as_ref() {
                    self.image_provider.get_or_load(ImageConfig::Alpaca {
                        resource_key: orm_resource_key.value.to_string(),
                    })
                } else {
                    self.default_orm_image.clone()
                };

                self.upload_material(*id, MaterialGPU::create(
                    archived_material_data.base_color_factor.map(|v| v.into()),
                    archived_material_data.roughness_factor.into(),
                    archived_material_data.metallic_factor.into(),
                    color_image.id,
                    normal_image.id,
                    orm_image.id,
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

                color_image,
                normal_image,
                orm_image,
            } => {
                self.upload_material(*id, MaterialGPU::create(
                    base_color_factor,
                    roughness_factor,
                    metallic_factor,
                    color_image.id,
                    normal_image.id,
                    orm_image.id,
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

    fn erase(&self, _id: &ResourceId) -> Result<()> {
        // self.upload_material(*id, self.default_material)?;

        Ok(())
    }

    fn statistics(&self) -> Self::Statistics {
        ()
    }

    fn destroy_resource(&self, _resource: Self::Output) -> Result<()> {
        Ok(())
    }

    fn destroy(self) -> Result<()> {
        self.resource_factories.buffer_factory.destroy_buffer(self.material_buffer.into_managed_buffer())?;
        
        Ok(())
    }
}
