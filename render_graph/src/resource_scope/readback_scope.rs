use std::collections::HashMap;
use std::mem::size_of;
use std::slice::from_raw_parts;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{cast_slice, Pod};
use gpu::{BufferBuilder, BufferInfo, ManagedBufferFactory};
use gpu_allocator::MemoryLocation;
use index_allocator::{FrameIndex, SliceIndex};
use crate::resource_scope::readback_entry::ReadbackEntry;
use crate::virtual_readback::physical_readback::PhysicalReadback;
use crate::virtual_readback::virtual_readback::VirtualReadback;

pub struct ReadbackScope {
    entries: Vec<ReadbackEntry>,
    handles: HashMap<&'static str, u32>,

    frame_index: FrameIndex,
}

impl ReadbackScope {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            handles: HashMap::new(),

            frame_index: FrameIndex::ZERO,
        }
    }

    pub fn create_readback<T: Pod>(
        &mut self,
        buffer_factory: &ManagedBufferFactory,
        label: &'static str,
        capacity: u32,
        frame_count: u32,
    ) -> Result<VirtualReadback<T>> {
        if let Some(&handle) = self.handles.get(label) {
            return Ok(VirtualReadback::new(handle));
        }

        let frame_size = (size_of::<T>() * capacity.max(1) as usize) as DeviceSize;

        let buffer = BufferBuilder::slice::<u8>(frame_size as u32)
            .per_frame(frame_count)
            .build(
                buffer_factory,
                label,
                BufferUsageFlags::STORAGE_BUFFER
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                    | BufferUsageFlags::TRANSFER_DST,
                MemoryLocation::GpuToCpu,
            )?;

        let handle = self.entries.len() as u32;

        self.handles.insert(label, handle);
        self.entries.push(ReadbackEntry { buffer, frame_size });

        Ok(VirtualReadback::new(handle))
    }

    pub fn get_physical_readback<T: Pod>(&self, readback: VirtualReadback<T>) -> PhysicalReadback {
        self.physical(readback.handle)
    }

    pub fn values<T: Pod>(&self, readback: VirtualReadback<T>) -> &[T] {
        let physical = self.physical(readback.handle);

        let bytes = unsafe { from_raw_parts(physical.mapped_ptr, physical.size as usize) };

        cast_slice(bytes)
    }

    pub fn begin_frame(&mut self, frame_index: FrameIndex) {
        self.frame_index = frame_index;
    }

    pub fn physical_readbacks(&self) -> impl Iterator<Item = PhysicalReadback> + '_ {
        (0..self.entries.len() as u32).map(|handle| self.physical(handle))
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        for entry in self.entries {
            buffer_factory.destroy_buffer(entry.buffer.into_managed_buffer())?;
        }

        Ok(())
    }

    fn physical(&self, handle: u32) -> PhysicalReadback {
        let entry = &self.entries[handle as usize];

        let view = entry.buffer.frame(self.frame_index).slice_at(SliceIndex::ZERO);

        PhysicalReadback {
            buffer: view.handle(),
            offset: view.offset(),
            size: entry.frame_size,
            device_address: view.device_address(),
            mapped_ptr: view.mapped_ptr(),
        }
    }
}
