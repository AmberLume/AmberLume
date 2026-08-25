use std::mem::take;
use anyhow::bail;
use anyhow::Result;
use ash::vk::{Buffer, BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;
use index_allocator::FrameIndex;
use crate::factories::buffer::block_heap::block_heap_configuration::BlockHeapConfiguration;
use crate::factories::buffer::block_heap::block_heap_statistics::BlockHeapStatistics;
use crate::factories::buffer::buffer_range::buffer_range::BufferRange;
use crate::factories::buffer::managed_buffer::ManagedBuffer;
use crate::factories::buffer::managed_buffer_factory::ManagedBufferFactory;

pub struct BlockHeap {
    name: &'static str,
    block_size: DeviceSize,
    usage: BufferUsageFlags,
    location: MemoryLocation,

    blocks: Vec<ManagedBuffer>,
    free_blocks: Vec<usize>,
    frame_blocks: Vec<Vec<usize>>,
    frame_oversize: Vec<Vec<ManagedBuffer>>,

    current_block: Option<usize>,
    head: DeviceSize,

    frame_index: FrameIndex,

    used: DeviceSize,
    peak_used: DeviceSize,
    oversize_count: u32,
}

impl BlockHeap {
    pub const ALIGNMENT: DeviceSize = 16;

    pub fn create(configuration: BlockHeapConfiguration) -> Result<Self> {
        if !configuration.block_size.is_power_of_two() {
            bail!(
                "Block heap '{}' block size {} is not a power of two",
                configuration.name, configuration.block_size,
            )
        }

        Ok(Self {
            name: configuration.name,
            block_size: configuration.block_size,
            usage: configuration.usage,
            location: configuration.location,

            blocks: Vec::new(),
            free_blocks: Vec::new(),
            frame_blocks: (0..configuration.frame_count).map(|_| Vec::new()).collect(),
            frame_oversize: (0..configuration.frame_count).map(|_| Vec::new()).collect(),

            current_block: None,
            head: 0,

            frame_index: FrameIndex::ZERO,

            used: 0,
            peak_used: 0,
            oversize_count: 0,
        })
    }

    pub fn begin_frame(
        &mut self,
        buffer_factory: &ManagedBufferFactory,
        frame_index: FrameIndex,
    ) -> Result<()> {
        if let Some(block) = self.current_block.take() {
            self.frame_blocks[self.frame_index.value as usize].push(block);
        }

        let slot = frame_index.value as usize;

        let retired = take(&mut self.frame_blocks[slot]);
        self.free_blocks.extend(retired);

        for oversize in take(&mut self.frame_oversize[slot]) {
            buffer_factory.destroy_buffer(oversize)?;
        }

        self.frame_index = frame_index;
        self.head = 0;
        self.used = 0;

        Ok(())
    }

    pub fn allocate(
        &mut self,
        buffer_factory: &ManagedBufferFactory,
        label: &'static str,
        size: DeviceSize,
        alignment: DeviceSize,
    ) -> Result<BufferRange> {
        let alignment = alignment.max(Self::ALIGNMENT);
        let size = size.max(Self::ALIGNMENT);

        if !alignment.is_power_of_two() {
            bail!("Alignment {} is not a power of two", alignment)
        }

        self.used += size;
        self.peak_used = self.peak_used.max(self.used);

        if size + alignment > self.block_size {
            return self.allocate_oversize(buffer_factory, label, size, alignment);
        }

        if let Some(block) = self.current_block {
            if let Some((range, head)) = Self::sub_range(&self.blocks[block], label, self.head, size, alignment)? {
                self.head = head;

                return Ok(range);
            }

            self.frame_blocks[self.frame_index.value as usize].push(block);
            self.current_block = None;
        }

        let block = match self.free_blocks.pop() {
            Some(block) => block,
            None => self.create_block(buffer_factory, self.block_size)?,
        };

        self.current_block = Some(block);

        let Some((range, head)) = Self::sub_range(&self.blocks[block], label, 0, size, alignment)? else {
            bail!(
                "Block heap '{}' cannot fit {} bytes aligned to {} into a block of {}",
                self.name, size, alignment, self.block_size,
            )
        };

        self.head = head;

        Ok(range)
    }

    pub fn buffers(&self) -> impl Iterator<Item = Buffer> + '_ {
        self.blocks.iter()
            .map(|block| block.handle)
            .chain(
                self.frame_oversize.iter()
                    .flat_map(|oversize| oversize.iter().map(|block| block.handle)),
            )
    }

    pub fn statistics(&self) -> BlockHeapStatistics {
        BlockHeapStatistics {
            block_size: self.block_size as u32,
            block_count: self.blocks.len() as u32,
            oversize_count: self.oversize_count,
            used: self.used as u32,
            peak_used: self.peak_used as u32,
            capacity: (self.blocks.len() as DeviceSize * self.block_size) as u32,
        }
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        for block in self.blocks {
            buffer_factory.destroy_buffer(block)?;
        }

        for oversize in self.frame_oversize.into_iter().flatten() {
            buffer_factory.destroy_buffer(oversize)?;
        }

        Ok(())
    }

    fn allocate_oversize(
        &mut self,
        buffer_factory: &ManagedBufferFactory,
        label: &'static str,
        size: DeviceSize,
        alignment: DeviceSize,
    ) -> Result<BufferRange> {
        let buffer = buffer_factory.create_managed_buffer(
            self.name,
            size + alignment,
            self.usage,
            self.location,
        )?;

        let Some((range, _)) = Self::sub_range(&buffer, label, 0, size, alignment)? else {
            bail!(
                "Block heap '{}' oversize block of {} cannot fit {} bytes aligned to {}",
                self.name, size + alignment, size, alignment,
            )
        };

        self.frame_oversize[self.frame_index.value as usize].push(buffer);
        self.oversize_count += 1;

        Ok(range)
    }

    fn create_block(
        &mut self,
        buffer_factory: &ManagedBufferFactory,
        size: DeviceSize,
    ) -> Result<usize> {
        let buffer = buffer_factory.create_managed_buffer(
            self.name,
            size,
            self.usage,
            self.location,
        )?;

        self.blocks.push(buffer);

        Ok(self.blocks.len() - 1)
    }

    fn sub_range(
        buffer: &ManagedBuffer,
        label: &'static str,
        head: DeviceSize,
        size: DeviceSize,
        alignment: DeviceSize,
    ) -> Result<Option<(BufferRange, DeviceSize)>> {
        let offset = Self::align_up(buffer.device_address + head, alignment) - buffer.device_address;
        let end = offset + size;

        if end > buffer.size {
            return Ok(None);
        }

        Ok(Some((buffer.range(label, offset, size)?, end)))
    }

    fn align_up(value: DeviceSize, alignment: DeviceSize) -> DeviceSize {
        (value + alignment - 1) & !(alignment - 1)
    }
}
