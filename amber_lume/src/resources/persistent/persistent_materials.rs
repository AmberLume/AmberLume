use std::sync::Arc;
use anyhow::Result;
use crate::ids::SliceIndex;
use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::buffer::typed::materials_buffer::MaterialGpuData;
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_manager::IndexManager;
use crate::resources::dynamic::resource_provider::ResourceId;
use crate::resources::persistent::persistent_images::PersistentImages;

pub struct PersistentMaterials {
    pub default: (ResourceId, MaterialGpuData),
}

impl PersistentMaterials {
    pub fn create(
        resource_loader: Arc<ResourceLoader>,
        material_index_manager: &IndexManager,
        buffer_manager: &BufferManager,
        persistent_images: &PersistentImages,
    ) -> Result<Self> {
        let default_resource_id = material_index_manager.acquire().unwrap();
        let default_data = MaterialGpuData::create(
            [1.0, 0.0, 1.0, 1.0],
            1.0,
            1.0,
            persistent_images.white_pixel.descriptor_index,
            persistent_images.default_normal.descriptor_index,
            persistent_images.default_occlusion_roughness_metallic.descriptor_index,
        );
        resource_loader.load_buffer_at(
            &buffer_manager.material_buffer.slice_at(SliceIndex { value: default_resource_id }),
            &[default_data],
        )?;

        Ok(Self {
            default: (default_resource_id, default_data),
        })
    }
}
