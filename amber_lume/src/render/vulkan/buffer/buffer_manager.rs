use anyhow::Result;
use tracing::info;
use yakui::paint::Vertex;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::vulkan::buffer::typed::culling_views_buffer::{create_culling_views_buffer, CullingViewGpuData};
use crate::render::vulkan::buffer::typed::draw_data_buffer::{create_draw_data_buffer, DrawDataGpuData};
use crate::render::vulkan::buffer::typed::draw_count_buffer::create_draw_count_buffer;
use crate::render::vulkan::buffer::typed::entity_buffer::{create_entity_buffer, EntityGpuData};
use crate::render::vulkan::buffer::typed::frame_stats_buffer::create_render_stats_buffer;
use crate::render::vulkan::buffer::typed::index_buffer::create_index_buffer;
use crate::render::vulkan::buffer::typed::indirect_buffer::{create_indirect_buffer, IndirectGpuData};
use crate::render::vulkan::buffer::typed::material_buffer::{create_material_buffer, MaterialGpuData};
use crate::render::vulkan::buffer::typed::model_buffer::{create_model_buffer, ModelGpuData};
use crate::render::vulkan::buffer::typed::renderer_staging_buffer::create_renderer_staging_buffer;
use crate::render::vulkan::buffer::typed::scene_buffer::{create_scene_buffer, SceneGpuData};
use crate::render::vulkan::buffer::typed::submesh_buffer::{create_submesh_buffer, SubmeshGpuData};
use crate::render::vulkan::buffer::typed::ui_index_buffer::create_ui_index_buffer;
use crate::render::vulkan::buffer::typed::ui_vertex_buffer::create_ui_vertex_buffer;
use crate::render::vulkan::buffer::typed::vertex_buffer::{create_vertex_buffer, VertexGpuData};
use crate::render::vulkan::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::vulkan::factories::buffer::chunk_buffer::chunk_buffer::ChunkBuffer;
use crate::render::vulkan::factories::buffer::flat_buffer::flat_buffer::FlatBuffer;
use crate::render::vulkan::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::vulkan::factories::buffer::typed_buffer::typed_buffer::TypedBuffer;
use crate::render::statistics::raw::raw_gpu_render_statistics::RawGpuRenderStatistics;

pub struct BufferManager {
    pub culling_views_buffer: FrameBuffer<SliceBuffer<CullingViewGpuData>>,

    pub indirect_buffer: ChunkBuffer<SliceBuffer<IndirectGpuData>>,
    pub draw_count_buffer: ChunkBuffer<TypedBuffer<u32>>,

    pub index_buffer: SliceBuffer<u32>,
    pub vertex_buffer: SliceBuffer<VertexGpuData>,

    pub ui_index_buffer: FrameBuffer<SliceBuffer<u32>>,
    pub ui_vertex_buffer: FrameBuffer<SliceBuffer<Vertex>>,

    pub submesh_buffer: SliceBuffer<SubmeshGpuData>,
    pub material_buffer: SliceBuffer<MaterialGpuData>,
    pub model_buffer: SliceBuffer<ModelGpuData>,

    pub entity_buffer: FrameBuffer<SliceBuffer<EntityGpuData>>,
    pub draw_data_buffer: ChunkBuffer<SliceBuffer<DrawDataGpuData>>,

    pub scene_buffer: FrameBuffer<TypedBuffer<SceneGpuData>>,

    pub renderer_staging_buffer: FrameBuffer<FlatBuffer>,

    pub render_stats_buffer: FrameBuffer<TypedBuffer<RawGpuRenderStatistics>>,
}

impl BufferManager {
    pub fn create(
        buffer_factory: &ManagedBufferFactory,
        renderer_limits: &RendererLimits,
    ) -> Result<Self> {
        let frames_in_flight = renderer_limits.frames_in_flight;

        let culling_views_buffer = create_culling_views_buffer(
            &buffer_factory,
            frames_in_flight,
            renderer_limits.render_resource_limits.max_render_views,
        )?;

        let indirect_buffer = create_indirect_buffer(
            &buffer_factory,
            renderer_limits.render_resource_limits.max_render_views,
            renderer_limits.render_resource_limits.max_draw_calls,
        )?;
        let draw_count_buffer = create_draw_count_buffer(
            &buffer_factory, 
            renderer_limits.render_resource_limits.max_render_views,
        )?;

        let index_buffer = create_index_buffer(
            &buffer_factory,
            renderer_limits.render_resource_limits.max_indices,
        )?;
        let vertex_buffer = create_vertex_buffer(
            &buffer_factory,
            renderer_limits.render_resource_limits.max_vertices,
        )?;

        let ui_index_buffer = create_ui_index_buffer(
            &buffer_factory,
            frames_in_flight,
            4096,
        )?;
        let ui_vertex_buffer = create_ui_vertex_buffer(
            &buffer_factory,
            frames_in_flight,
            4096,
        )?;

        let submesh_buffer = create_submesh_buffer(
            &buffer_factory, 
            renderer_limits.render_resource_limits.max_submeshes,
        )?;
        let material_buffer = create_material_buffer(
            &buffer_factory, 
            renderer_limits.render_resource_limits.max_materials,
        )?;
        let model_buffer = create_model_buffer(
            &buffer_factory, 
            renderer_limits.render_resource_limits.max_models,
        )?;

        let entity_buffer = create_entity_buffer(
            &buffer_factory,
            frames_in_flight,
            renderer_limits.buffer_limits.max_entities,
        )?;
        let draw_data_buffer = create_draw_data_buffer(
            &buffer_factory,
            renderer_limits.render_resource_limits.max_render_views,
            renderer_limits.render_resource_limits.max_draw_calls,
        )?;
        
        let scene_buffer = create_scene_buffer(buffer_factory, frames_in_flight)?;

        let renderer_staging_buffer = create_renderer_staging_buffer(buffer_factory, frames_in_flight, 128 * 1024)?;

        let render_stats_buffer = create_render_stats_buffer(buffer_factory, frames_in_flight)?;

        Ok(Self {
            culling_views_buffer,

            indirect_buffer,
            draw_count_buffer,

            index_buffer,
            vertex_buffer,

            ui_index_buffer,
            ui_vertex_buffer,

            submesh_buffer,
            material_buffer,
            model_buffer,

            entity_buffer,
            draw_data_buffer,

            scene_buffer,

            renderer_staging_buffer,

            render_stats_buffer,
        })
    }

    pub fn destroy(self, managed_buffer_factory: &ManagedBufferFactory) -> Result<()> {
        managed_buffer_factory.destroy_buffer(self.render_stats_buffer.into_managed_buffer())?;

        managed_buffer_factory.destroy_buffer(self.renderer_staging_buffer.into_managed_buffer())?;

        managed_buffer_factory.destroy_buffer(self.scene_buffer.into_managed_buffer())?;

        managed_buffer_factory.destroy_buffer(self.draw_data_buffer.into_managed_buffer())?;
        managed_buffer_factory.destroy_buffer(self.entity_buffer.into_managed_buffer())?;

        managed_buffer_factory.destroy_buffer(self.model_buffer.into_managed_buffer())?;
        managed_buffer_factory.destroy_buffer(self.material_buffer.into_managed_buffer())?;
        managed_buffer_factory.destroy_buffer(self.submesh_buffer.into_managed_buffer())?;

        managed_buffer_factory.destroy_buffer(self.ui_vertex_buffer.into_managed_buffer())?;
        managed_buffer_factory.destroy_buffer(self.ui_index_buffer.into_managed_buffer())?;

        managed_buffer_factory.destroy_buffer(self.vertex_buffer.into_managed_buffer())?;
        managed_buffer_factory.destroy_buffer(self.index_buffer.into_managed_buffer())?;

        managed_buffer_factory.destroy_buffer(self.draw_count_buffer.into_managed_buffer())?;
        managed_buffer_factory.destroy_buffer(self.indirect_buffer.into_managed_buffer())?;

        managed_buffer_factory.destroy_buffer(self.culling_views_buffer.into_managed_buffer())?;

        info!("BufferManager destroyed");

        Ok(())
    }
}
