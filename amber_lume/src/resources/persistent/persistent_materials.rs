use std::sync::Arc;
use anyhow::Result;
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::buffer::typed::material_buffer::MaterialGpuData;
use crate::render::vulkan::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_manager::DescriptorIndexManager;
use crate::resources::dynamic::resource_provider::ResourceId;

pub struct PersistentMaterials {
    pub default: (ResourceId, MaterialGpuData),
}

impl PersistentMaterials {
    pub fn create(
        resource_loader: Arc<ResourceLoader>,
        material_index_manager: &DescriptorIndexManager,
        buffer_manager: &BufferManager,
    ) -> Result<Self> {
        let default_resource_id = material_index_manager.acquire().unwrap();
        let default_data = MaterialGpuData::create(
            [1.0, 0.0, 1.0, 1.0],
            !0,
        );

        resource_loader.load_buffer_at(
            &buffer_manager.material_buffer,
            default_resource_id,
            &[default_data],
        )?;

        Ok(Self {
            default: (default_resource_id, default_data),
        })
    }
}
