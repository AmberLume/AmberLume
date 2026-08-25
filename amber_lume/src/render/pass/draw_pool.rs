use anyhow::Result;
use ash::vk::DeviceSize;
use render_graph::BufferResourceScope;
use render_graph::VirtualBuffer;
use std::mem::size_of;
use render_graph::IndirectGPU;
use crate::render::frame_data::draw_data_buffer::DrawDataGPU;

#[derive(Copy, Clone)]
pub struct DrawPool {
    pub indirect: VirtualBuffer,
    pub draw_count: VirtualBuffer,
    pub draw_data: VirtualBuffer,

    pub capacity: u32,
}

impl DrawPool {
    pub const BUCKET_COUNT: u32 = 3;

    pub fn reserve(&self, buffer_scope: &mut BufferResourceScope) -> Result<()> {
        self.indirect.reserve_region(
            buffer_scope,
            self.capacity as DeviceSize * size_of::<IndirectGPU>() as DeviceSize,
        )?;
        self.draw_data.reserve_region(
            buffer_scope,
            self.capacity as DeviceSize * size_of::<DrawDataGPU>() as DeviceSize,
        )?;
        self.draw_count.reserve_region(
            buffer_scope,
            Self::BUCKET_COUNT as DeviceSize * size_of::<u32>() as DeviceSize,
        )?;

        Ok(())
    }
}
