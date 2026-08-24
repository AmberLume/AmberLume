use crate::factories::buffer::buffer_range::buffer_range::BufferRange;
use anyhow::bail;
use ash::vk::DeviceSize;
use anyhow::Result;
use ash::vk::{Extent3D, Image, ImageSubresourceLayers};
use bytemuck::{cast_slice, Pod};
use crossbeam_channel::{bounded, Sender};
use crate::buffer::transfer_context::TransferTask;

pub struct ResourceTransfer {
    pub transfer_tx: Sender<TransferTask>,
}

impl ResourceTransfer {
    pub fn create(
        transfer_tx: Sender<TransferTask>,
    ) -> Self {
        Self {
            transfer_tx,
        }
    }

    pub fn load_buffer_at<T: Pod>(
        &self,
        range: BufferRange,
        data: &[T],
    ) -> Result<()> {
        let data_slice = cast_slice(data).to_vec();

        if data_slice.len() as DeviceSize > range.size {
            bail!(
                "Upload of {} bytes into range '{}' exceeds size {}",
                data_slice.len(),
                range.label,
                range.size,
            )
        }

        self.transfer_tx.send(TransferTask::Buffer {
            handle: range.buffer,
            data: data_slice,
            offset: range.offset,
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

    pub fn flush_blocking(&self) -> Result<()> {
        let (acknowledge, completed) = bounded(1);

        self.transfer_tx.send(TransferTask::Flush { acknowledge })?;

        completed.recv()?;

        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        self.transfer_tx.send(TransferTask::Terminate)?;

        Ok(())
    }
}
