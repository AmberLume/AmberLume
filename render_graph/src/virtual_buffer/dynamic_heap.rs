use std::collections::HashMap;
use anyhow::bail;
use anyhow::Result;
use ash::vk::{Buffer, DeviceSize};
use gpu::BlockHeap;
use gpu::BlockHeapConfiguration;
use gpu::BlockHeapStatistics;
use gpu::BufferRange;
use gpu::ManagedBufferFactory;
use index_allocator::FrameIndex;
use crate::virtual_buffer::virtual_buffer::VirtualBuffer;

pub struct DynamicHeap {
    heap: BlockHeap,
    bindings: HashMap<VirtualBuffer, BufferRange>,
}

impl DynamicHeap {
    pub fn create(configuration: BlockHeapConfiguration) -> Result<Self> {
        Ok(Self {
            heap: BlockHeap::create(configuration)?,
            bindings: HashMap::new(),
        })
    }

    pub fn begin_frame(
        &mut self,
        buffer_factory: &ManagedBufferFactory,
        frame_index: FrameIndex,
    ) -> Result<()> {
        self.bindings.clear();

        self.heap.begin_frame(buffer_factory, frame_index)
    }

    pub fn upload(
        &mut self,
        buffer_factory: &ManagedBufferFactory,
        handle: VirtualBuffer,
        label: &'static str,
        size: DeviceSize,
        alignment: DeviceSize,
    ) -> Result<BufferRange> {
        if self.bindings.contains_key(&handle) {
            bail!("Dynamic buffer '{label}' is written twice in one frame")
        }

        let range = self.heap.allocate(buffer_factory, label, size, alignment)?;

        self.bindings.insert(handle, range);

        Ok(range)
    }

    pub fn reserve(
        &mut self,
        buffer_factory: &ManagedBufferFactory,
        handle: VirtualBuffer,
        label: &'static str,
        size: DeviceSize,
        alignment: DeviceSize,
    ) -> Result<BufferRange> {
        if let Some(range) = self.bindings.get(&handle) {
            if range.size != size.max(BlockHeap::ALIGNMENT) {
                bail!(
                    "Dynamic buffer '{label}' is reserved twice in one frame with {} and {} bytes",
                    range.size, size,
                )
            }

            return Ok(*range);
        }

        let range = self.heap.allocate(buffer_factory, label, size, alignment)?;

        self.bindings.insert(handle, range);

        Ok(range)
    }

    pub fn binding(&self, handle: VirtualBuffer) -> Option<BufferRange> {
        self.bindings.get(&handle).copied()
    }

    pub fn buffers(&self) -> impl Iterator<Item = Buffer> + '_ {
        self.heap.buffers()
    }

    pub fn statistics(&self) -> BlockHeapStatistics {
        self.heap.statistics()
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        self.heap.destroy(buffer_factory)
    }
}
