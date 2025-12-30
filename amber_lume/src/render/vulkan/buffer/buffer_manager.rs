use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::{bail, Result};
use std::sync::Arc;
use tracing::info;
use crate::render::vulkan::buffer::typed::draw_buffer::create_draw_buffer;
use crate::render::vulkan::buffer::typed::draw_count_buffer::create_draw_count_buffer;
use crate::render::vulkan::buffer::typed::entity_buffer::create_entity_buffer;
use crate::render::vulkan::buffer::typed::index_buffer::create_index_buffer;
use crate::render::vulkan::buffer::typed::indirect_buffer::create_indirect_buffer;
use crate::render::vulkan::buffer::typed::material_buffer::create_material_buffer;
use crate::render::vulkan::buffer::typed::model_buffer::create_model_buffer;
use crate::render::vulkan::buffer::typed::primitive_buffer::create_primitive_buffer;
use crate::render::vulkan::buffer::typed::resource_availability_buffer::create_resource_availability_buffer;
use crate::render::vulkan::buffer::typed::scene_buffer::create_scene_buffer;
use crate::render::vulkan::buffer::typed::vertex_buffer::create_vertex_buffer;

pub struct BufferManager {
    pub indirect_buffer: Arc<Buffer>,
    pub draw_count_buffer: Arc<Buffer>,

    pub scene_buffer: Arc<Buffer>,

    pub index_buffer: Arc<Buffer>,
    pub vertex_buffer: Arc<Buffer>,

    pub entity_buffer: Arc<Buffer>,

    pub model_buffer: Arc<Buffer>,
    pub model_availability_buffer: Arc<Buffer>,

    pub material_buffer: Arc<Buffer>,
    pub material_availability_buffer: Arc<Buffer>,

    pub image_availability_buffer: Arc<Buffer>,

    pub primitive_buffer: Arc<Buffer>,
    
    pub draw_buffer: Arc<Buffer>,
}

impl BufferManager {
    pub fn create(device_context: &mut DeviceContext) -> Self {
        let indirect_buffer = create_indirect_buffer(device_context, 1_000_000).unwrap();
        let draw_count_buffer = create_draw_count_buffer(device_context).unwrap();

        let scene_buffer = create_scene_buffer(device_context).unwrap();

        let index_buffer = create_index_buffer(device_context, 500_000).unwrap();
        let vertex_buffer = create_vertex_buffer(device_context, 1_500_000).unwrap();

        let entity_buffer = create_entity_buffer(device_context, 1_000_000).unwrap();

        let model_buffer = create_model_buffer(device_context, 10_000).unwrap();
        let model_availability_buffer = create_resource_availability_buffer(device_context, "model", 10_000).unwrap();

        let primitive_buffer = create_primitive_buffer(device_context, 100_000).unwrap();

        let material_buffer = create_material_buffer(device_context, 10_000).unwrap();
        let material_availability_buffer = create_resource_availability_buffer(device_context, "material", 10_000).unwrap();

        let image_availability_buffer = create_resource_availability_buffer(device_context, "image", 10_000).unwrap();

        let draw_buffer = create_draw_buffer(device_context, 1_000_000).unwrap();
        
        Self {
            indirect_buffer: Arc::new(indirect_buffer),
            draw_count_buffer: Arc::new(draw_count_buffer),

            scene_buffer: Arc::new(scene_buffer),

            index_buffer: Arc::new(index_buffer),
            vertex_buffer: Arc::new(vertex_buffer),

            entity_buffer: Arc::new(entity_buffer),

            model_buffer: Arc::new(model_buffer),
            model_availability_buffer: Arc::new(model_availability_buffer),

            material_buffer: Arc::new(material_buffer),
            material_availability_buffer: Arc::new(material_availability_buffer),

            image_availability_buffer: Arc::new(image_availability_buffer),
            
            primitive_buffer: Arc::new(primitive_buffer),
            
            draw_buffer: Arc::new(draw_buffer),
        }
    }

    pub fn destroy(self) -> Result<()> {
        Self::try_destroy_buffer(self.draw_buffer)?;
        
        Self::try_destroy_buffer(self.primitive_buffer)?;
        
        Self::try_destroy_buffer(self.image_availability_buffer)?;

        Self::try_destroy_buffer(self.material_availability_buffer)?;
        Self::try_destroy_buffer(self.material_buffer)?;

        Self::try_destroy_buffer(self.model_availability_buffer)?;
        Self::try_destroy_buffer(self.model_buffer)?;

        Self::try_destroy_buffer(self.entity_buffer)?;

        Self::try_destroy_buffer(self.vertex_buffer)?;
        Self::try_destroy_buffer(self.index_buffer)?;

        Self::try_destroy_buffer(self.scene_buffer)?;

        Self::try_destroy_buffer(self.draw_count_buffer)?;
        Self::try_destroy_buffer(self.indirect_buffer)?;

        info!("BufferManager destroyed");

        Ok(())
    }

    fn try_destroy_buffer(buffer: Arc<Buffer>) -> Result<()> {
        match Arc::try_unwrap(buffer) {
            Ok(mut buffer) => buffer.destroy(),
            Err(buffer) => {
                bail!("Can't destroy buffer '{}', still in use", buffer.name)
            }
        }
    }
}
