use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;
use anyhow::Result;
use tracing::info;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::vulkan::buffer::typed::culling_views_buffer::create_culling_views_buffer;
use crate::render::vulkan::buffer::typed::draw_buffer::create_draw_data_buffer;
use crate::render::vulkan::buffer::typed::draw_count_buffer::create_draw_count_buffer;
use crate::render::vulkan::buffer::typed::entity_buffer::create_entity_buffer;
use crate::render::vulkan::buffer::typed::index_buffer::create_index_buffer;
use crate::render::vulkan::buffer::typed::indirect_buffer::create_indirect_buffer;
use crate::render::vulkan::buffer::typed::material_buffer::create_material_buffer;
use crate::render::vulkan::buffer::typed::model_buffer::create_model_buffer;
use crate::render::vulkan::buffer::typed::submesh_buffer::create_submesh_buffer;
use crate::render::vulkan::buffer::typed::ui_index_buffer::create_ui_index_buffer;
use crate::render::vulkan::buffer::typed::ui_vertex_buffer::create_ui_vertex_buffer;
use crate::render::vulkan::buffer::typed::vertex_buffer::create_vertex_buffer;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;

pub struct BufferManager {
    pub culling_views_buffer: PoolBuffer,

    pub indirect_buffer: PoolBuffer,
    pub draw_count_buffer: PoolBuffer,

    pub index_buffer: PoolBuffer,
    pub vertex_buffer: PoolBuffer,

    pub ui_index_buffer: PoolBuffer,
    pub ui_vertex_buffer: PoolBuffer,

    pub entity_buffer: PoolBuffer,

    pub model_buffer: PoolBuffer,

    pub material_buffer: PoolBuffer,

    pub submesh_buffer: PoolBuffer,
    
    pub draw_data_buffer: PoolBuffer,
}

impl BufferManager {
    pub fn create(
        buffer_factory: &ManagedBufferFactory,
        renderer_limits: &RendererLimits,
    ) -> Self {
        let culling_views_buffer = create_culling_views_buffer(buffer_factory, renderer_limits.max_views).unwrap();

        let indirect_buffer = create_indirect_buffer(buffer_factory, renderer_limits.max_draw_calls, renderer_limits.max_views).unwrap();
        let draw_count_buffer = create_draw_count_buffer(buffer_factory, renderer_limits.max_views).unwrap();

        let index_buffer = create_index_buffer(buffer_factory, renderer_limits.max_indices).unwrap();
        let vertex_buffer = create_vertex_buffer(buffer_factory, renderer_limits.max_vertices).unwrap();

        let ui_index_buffer = create_ui_index_buffer(buffer_factory, 64 * 1024).unwrap();
        let ui_vertex_buffer = create_ui_vertex_buffer(buffer_factory, 128 * 1024).unwrap();

        let entity_buffer = create_entity_buffer(buffer_factory, renderer_limits.max_entities).unwrap();

        let model_buffer = create_model_buffer(buffer_factory, renderer_limits.max_models).unwrap();

        let submesh_buffer = create_submesh_buffer(buffer_factory, renderer_limits.max_submeshes).unwrap();

        let material_buffer = create_material_buffer(buffer_factory, renderer_limits.max_materials).unwrap();

        let draw_data_buffer = create_draw_data_buffer(buffer_factory, renderer_limits.max_draw_calls, renderer_limits.max_views).unwrap();
        
        Self {
            culling_views_buffer,

            indirect_buffer,
            draw_count_buffer,

            index_buffer,
            vertex_buffer,

            ui_index_buffer,
            ui_vertex_buffer,

            entity_buffer,

            model_buffer,

            material_buffer,
            
            submesh_buffer,
            
            draw_data_buffer,
        }
    }

    pub fn destroy(self, managed_buffer_factory: &ManagedBufferFactory) -> Result<()> {
        managed_buffer_factory.destroy_buffer(self.draw_data_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.submesh_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.material_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.model_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.entity_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.ui_vertex_buffer.handle)?;
        managed_buffer_factory.destroy_buffer(self.ui_index_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.vertex_buffer.handle)?;
        managed_buffer_factory.destroy_buffer(self.index_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.draw_count_buffer.handle)?;
        managed_buffer_factory.destroy_buffer(self.indirect_buffer.handle)?;

        managed_buffer_factory.destroy_buffer(self.culling_views_buffer.handle)?;

        info!("BufferManager destroyed");

        Ok(())
    }
}
