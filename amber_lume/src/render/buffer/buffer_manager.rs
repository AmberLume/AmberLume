use anyhow::Result;
use tracing::info;
use crate::limits::ResourceLimits;
use crate::render::buffer::typed::renderer_staging_buffer::create_renderer_staging_buffer;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::flat_buffer::flat_buffer::FlatBuffer;
use crate::render::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;

pub struct BufferManager {
    pub renderer_staging_buffer: FrameBuffer<FlatBuffer>,
}

impl BufferManager {
    pub fn create(
        buffer_factory: &ManagedBufferFactory,
        _limits: &ResourceLimits,
        frame_count: u32,
    ) -> Result<Self> {
        let renderer_staging_buffer = create_renderer_staging_buffer(buffer_factory, frame_count, 128 * 1024)?;

        Ok(Self {
            renderer_staging_buffer,
        })
    }

    pub fn destroy(self, managed_buffer_factory: &ManagedBufferFactory) -> Result<()> {
        managed_buffer_factory.destroy_buffer(self.renderer_staging_buffer.into_managed_buffer())?;

        info!("BufferManager destroyed");

        Ok(())
    }
}
