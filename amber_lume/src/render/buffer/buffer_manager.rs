use anyhow::Result;
use tracing::info;
use crate::limits::ResourceLimits;
use crate::render::buffer::typed::draw_data_buffer::{create_draw_data_buffer, DrawDataGPU};
use crate::render::buffer::typed::draw_count_buffer::create_draw_count_buffer;
use crate::render::buffer::typed::indirect_buffer::{create_indirect_buffer, IndirectGPU};
use crate::render::buffer::typed::renderer_staging_buffer::create_renderer_staging_buffer;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::chunk_buffer::chunk_buffer::ChunkBuffer;
use crate::render::factories::buffer::flat_buffer::flat_buffer::FlatBuffer;
use crate::render::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::buffer::typed_buffer::typed_buffer::TypedBuffer;

pub struct BufferManager {
    pub indirect_buffer: ChunkBuffer<SliceBuffer<IndirectGPU>>,
    pub draw_count_buffer: ChunkBuffer<TypedBuffer<u32>>,

    pub draw_data_buffer: ChunkBuffer<SliceBuffer<DrawDataGPU>>,

    pub renderer_staging_buffer: FrameBuffer<FlatBuffer>,
}

impl BufferManager {
    pub fn create(
        buffer_factory: &ManagedBufferFactory,
        limits: &ResourceLimits,
        frame_count: u32,
    ) -> Result<Self> {
        let indirect_buffer = create_indirect_buffer(
            &buffer_factory,
            limits.max_render_views,
            limits.max_draw_calls,
        )?;
        let draw_count_buffer = create_draw_count_buffer(
            &buffer_factory,
            limits.max_render_views,
        )?;

        let draw_data_buffer = create_draw_data_buffer(
            &buffer_factory,
            limits.max_render_views,
            limits.max_draw_calls,
        )?;
        
        let renderer_staging_buffer = create_renderer_staging_buffer(buffer_factory, frame_count, 128 * 1024)?;

        Ok(Self {
            indirect_buffer,
            draw_count_buffer,

            draw_data_buffer,

            renderer_staging_buffer,
        })
    }

    pub fn destroy(self, managed_buffer_factory: &ManagedBufferFactory) -> Result<()> {
        managed_buffer_factory.destroy_buffer(self.renderer_staging_buffer.into_managed_buffer())?;

        managed_buffer_factory.destroy_buffer(self.draw_data_buffer.into_managed_buffer())?;

        managed_buffer_factory.destroy_buffer(self.draw_count_buffer.into_managed_buffer())?;
        managed_buffer_factory.destroy_buffer(self.indirect_buffer.into_managed_buffer())?;

        info!("BufferManager destroyed");

        Ok(())
    }
}
