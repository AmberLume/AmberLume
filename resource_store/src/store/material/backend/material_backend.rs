use gpu_data::MaterialGPU;
use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;
use gpu::ResourceTransfer;
use resource_residency::ResourceBackend;
use index_allocator::ResourceId;
use gpu::SingleAllocation;
use crate::store::material::backend::managed_material::ManagedMaterial;
use crate::store::material::backend::material_config::MaterialConfig;

pub struct MaterialBackend {
    resource_transfer: Arc<ResourceTransfer>,

    material: Arc<SingleAllocation<MaterialGPU>>,

    materials: DashMap<ResourceId, ManagedMaterial>,
}

impl MaterialBackend {
    pub(crate) fn new(
        resource_transfer: Arc<ResourceTransfer>,
        material: Arc<SingleAllocation<MaterialGPU>>,
    ) -> Self {
        Self {
            resource_transfer,

            material,

            materials: DashMap::new(),
        }
    }

    fn upload_material(&self, id: ResourceId, data: MaterialGPU) -> Result<()> {
        self.resource_transfer.load_buffer_at(
            self.material.at(id.inner),
            &[data],
        )
    }
}

impl ResourceBackend for MaterialBackend {
    type Config = MaterialConfig;
    type Output = ();
    type Statistics = ();

    fn reserve(&self, id: &ResourceId) -> Result<()> {
        self.erase(id)
    }

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        let MaterialConfig {
            base_color_factor,
            roughness_factor,
            metallic_factor,

            alpha_mode,
            alpha_cutoff,

            color_image,
            normal_image,
            orm_image,
        } = config;

        let record = MaterialGPU::create(
            base_color_factor,
            roughness_factor,
            metallic_factor,
            alpha_mode,
            alpha_cutoff,
            color_image.id.inner,
            normal_image.id.inner,
            orm_image.id.inner,
        );

        self.upload_material(*id, record)?;

        self.materials.insert(*id, ManagedMaterial {
            images: vec![
                color_image,
                normal_image,
                orm_image,
            ],
        });

        info!("Uploaded material: index: {}, data: {:?}", id.inner, record);

        Ok(())
    }

    fn erase(&self, id: &ResourceId) -> Result<()> {
        if let Some((_, material)) = self.materials.remove(id) {
            info!(
                "Destroyed material: index: {}, images: {:?}",
                id.inner, material.images.iter().map(|image| image.id.inner).collect::<Vec<_>>(),
            );
        }

        self.upload_material(*id, MaterialGPU::DEFAULT)
    }

    fn statistics(&self) -> Self::Statistics {
        ()
    }
}
