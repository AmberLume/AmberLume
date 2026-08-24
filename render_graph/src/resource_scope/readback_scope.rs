use std::collections::HashMap;
use std::mem::size_of;
use std::ptr::write_bytes;
use std::slice::from_raw_parts;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{cast_slice, from_bytes, Pod};
use gpu::{FrameRegions, ManagedBufferFactory};
use gpu_allocator::MemoryLocation;
use index_allocator::FrameIndex;
use crate::resource_scope::readback_entry::ReadbackEntry;
use crate::virtual_readback::physical_readback::PhysicalReadback;
use crate::virtual_readback::readback_header::ReadbackHeader;
use crate::virtual_readback::virtual_readback::VirtualReadback;

const SLOT_ALIGNMENT: DeviceSize = 16;

fn slot_size(size: DeviceSize) -> DeviceSize {
    size.next_multiple_of(SLOT_ALIGNMENT)
}

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

        let slot = (size_of::<ReadbackHeader>() + size_of::<T>() * capacity.max(1) as usize) as DeviceSize;
        let frame_size = slot_size(slot);

        let allocation = buffer_factory.create_managed_buffer(
            label,
            frame_size * frame_count as DeviceSize,
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuToCpu,
        )?;
        let frames = FrameRegions::create(allocation.whole(label), frame_size, frame_count);

        for frame in 0..frame_count {
            let range = frames.frame(FrameIndex::from(frame));

            unsafe { write_bytes(range.mapped_ptr, 0, frame_size as usize) };
        }

        let handle = self.entries.len() as u32;

        self.handles.insert(label, handle);
        self.entries.push(ReadbackEntry {
            allocation,
            frames,

            snapshot: vec![0; slot as usize],
        });

        Ok(VirtualReadback::new(handle))
    }

    pub fn get_physical_readback<T: Pod>(&self, readback: VirtualReadback<T>) -> PhysicalReadback {
        self.physical(readback.handle)
    }

    pub fn value<T: Pod>(&self, readback: VirtualReadback<T>) -> Option<&T> {
        self.values(readback).map(|values| &values[0])
    }

    pub fn values<T: Pod>(&self, readback: VirtualReadback<T>) -> Option<&[T]> {
        let (header, payload) = self.entries[readback.handle as usize]
            .snapshot
            .split_at(size_of::<ReadbackHeader>());

        if from_bytes::<ReadbackHeader>(header).written == 0 {
            return None;
        }

        Some(cast_slice(payload))
    }

    pub fn begin_frame(&mut self, frame_index: FrameIndex) {
        self.frame_index = frame_index;

        for handle in 0..self.entries.len() {
            let physical = self.physical(handle as u32);

            let slot = self.entries[handle].snapshot.len();
            let mapped = unsafe { from_raw_parts(physical.range.mapped_ptr, slot) };

            self.entries[handle].snapshot.copy_from_slice(mapped);
        }
    }

    pub fn physical_readbacks(&self) -> impl Iterator<Item = PhysicalReadback> + '_ {
        (0..self.entries.len() as u32).map(|handle| self.physical(handle))
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        for entry in self.entries {
            buffer_factory.destroy_buffer(entry.allocation)?;
        }

        Ok(())
    }

    fn physical(&self, handle: u32) -> PhysicalReadback {
        let entry = &self.entries[handle as usize];

        let range = entry.frames.frame(self.frame_index)
            .sub(0, slot_size(entry.snapshot.len() as DeviceSize))
            .expect("Readback slot must fit its frame region");

        PhysicalReadback::create(range)
    }
}
