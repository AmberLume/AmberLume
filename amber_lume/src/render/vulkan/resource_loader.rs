use anyhow::Result;
use ash::vk::{Extent3D, Image, ImageSubresourceLayers};
use bytemuck::cast_slice;
use crossbeam_channel::Sender;
use crate::render::vulkan::buffer::transfer_context::TransferTask;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::vulkan::factories::buffer::view::buffer_view::BufferView;

pub struct ResourceLoader {
    pub transfer_tx: Sender<TransferTask>,
}

impl ResourceLoader {
    pub fn create(
        transfer_tx: Sender<TransferTask>,
    ) -> Self {
        Self {
            transfer_tx,
        }
    }

    pub fn load_buffer_at<T: bytemuck::Pod>(
        &self,
        buffer_view: &BufferView<ManagedBuffer>,
        data: &[T],
    ) -> Result<()> {
        let data_slice = cast_slice(data).to_vec();

        self.transfer_tx.send(TransferTask::Buffer {
            handle: buffer_view.handle(),
            data: data_slice,
            offset: buffer_view.offset(),
        })?;

        Ok(())
    }

    pub fn load_image(
        &self,
        image: Image,
        extent: Extent3D,
        subresource: ImageSubresourceLayers,
        level_count: u32,
        layer_count: u32,
        data: &[u8],
    ) -> Result<()> {
        self.transfer_tx.send(TransferTask::Image {
            handle: image,
            data: data.to_vec(),
            extent,
            subresource,
            level_count,
            layer_count,
        })?;

        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        self.transfer_tx.send(TransferTask::Terminate)?;

        Ok(())
    }
}
