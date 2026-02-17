use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;
use anyhow::Result;
use tracing::info;
use crate::render::vulkan::buffer::typed::collider_buffer::create_collider_buffer;
use crate::render::vulkan::buffer::typed::draw_buffer::create_draw_buffer;
use crate::render::vulkan::buffer::typed::draw_count_buffer::create_draw_count_buffer;
use crate::render::vulkan::buffer::typed::entity_buffer::create_entity_buffer;
use crate::render::vulkan::buffer::typed::index_buffer::create_index_buffer;
use crate::render::vulkan::buffer::typed::indirect_buffer::create_indirect_buffer;
use crate::render::vulkan::buffer::typed::indirect_non_indexed_buffer::create_collider_indirect_buffer;
use crate::render::vulkan::buffer::typed::material_buffer::create_material_buffer;
use crate::render::vulkan::buffer::typed::model_buffer::create_model_buffer;
use crate::render::vulkan::buffer::typed::primitive_buffer::create_primitive_buffer;
use crate::render::vulkan::buffer::typed::scene_buffer::create_scene_buffer;
use crate::render::vulkan::buffer::typed::ui_index_buffer::create_ui_index_buffer;
use crate::render::vulkan::buffer::typed::ui_vertex_buffer::create_ui_vertex_buffer;
use crate::render::vulkan::buffer::typed::vertex_buffer::create_vertex_buffer;
use crate::render::vulkan::factories::buffer::linear_buffer::LinearBuffer;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;

pub struct BufferManager {
    pub indirect_buffer: PoolBuffer,
    pub collider_indirect_buffer: PoolBuffer,
    pub draw_count_buffer: LinearBuffer,

    pub scene_buffer: LinearBuffer,

    pub index_buffer: PoolBuffer,
    pub vertex_buffer: PoolBuffer,

    pub ui_index_buffer: PoolBuffer,
    pub ui_vertex_buffer: PoolBuffer,

    pub entity_buffer: PoolBuffer,

    pub collider_buffer: PoolBuffer,

    pub model_buffer: PoolBuffer,

    pub material_buffer: PoolBuffer,

    pub primitive_buffer: PoolBuffer,
    
    pub draw_buffer: PoolBuffer,
}

impl BufferManager {
    pub fn create(
        buffer_factory: &ManagedBufferFactory,
    ) -> Self {
        let indirect_buffer = create_indirect_buffer(buffer_factory, 100_000).unwrap();
        let collider_indirect_buffer = create_collider_indirect_buffer(buffer_factory, 100_000).unwrap();
        let draw_count_buffer = create_draw_count_buffer(buffer_factory).unwrap();

        let scene_buffer = create_scene_buffer(buffer_factory).unwrap();

        let index_buffer = create_index_buffer(buffer_factory, 500_000).unwrap();
        let vertex_buffer = create_vertex_buffer(buffer_factory, 1_500_000).unwrap();

        let ui_index_buffer = create_ui_index_buffer(buffer_factory, 64 * 1024).unwrap();
        let ui_vertex_buffer = create_ui_vertex_buffer(buffer_factory, 128 * 1024).unwrap();

        let entity_buffer = create_entity_buffer(buffer_factory, 100_000).unwrap();

        let collider_buffer = create_collider_buffer(buffer_factory, 100_000).unwrap();
        
        let model_buffer = create_model_buffer(buffer_factory, 1000).unwrap();

        let primitive_buffer = create_primitive_buffer(buffer_factory, 50_000).unwrap();

        let material_buffer = create_material_buffer(buffer_factory, 10_000).unwrap();

        let draw_buffer = create_draw_buffer(buffer_factory, 100_000).unwrap();
        
        Self {
            indirect_buffer,
            collider_indirect_buffer,
            draw_count_buffer,

            scene_buffer,

            index_buffer,
            vertex_buffer,

            ui_index_buffer,
            ui_vertex_buffer,

            entity_buffer,

            collider_buffer,

            model_buffer,

            material_buffer,
            
            primitive_buffer,
            
            draw_buffer,
        }
    }

    pub fn destroy(self, managed_buffer_factory: &ManagedBufferFactory) -> Result<()> {
        managed_buffer_factory.destroy_buffer(self.draw_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.primitive_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.material_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.model_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.collider_buffer.handle)?;
        
        managed_buffer_factory.destroy_buffer(self.entity_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.ui_vertex_buffer.handle)?;
        managed_buffer_factory.destroy_buffer(self.ui_index_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.vertex_buffer.handle)?;
        managed_buffer_factory.destroy_buffer(self.index_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.scene_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.draw_count_buffer.handle)?;
        managed_buffer_factory.destroy_buffer(self.collider_indirect_buffer.handle)?;
        managed_buffer_factory.destroy_buffer(self.indirect_buffer.handle)?;

        info!("BufferManager destroyed");

        Ok(())
    }
}
