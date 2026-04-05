use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::resources::dynamic::skinning::bone_transform_buffer::{
    BoneTransformGPU, create_bone_transform_buffer,
};
use crate::resources::range_allocator::range_allocator::{Allocation, RangeAllocator};
use anyhow::Result;

pub struct BoneTransformHandler {
    allocator: RangeAllocator,
    pub(crate) buffer: SliceBuffer<BoneTransformGPU>,
}

impl BoneTransformHandler {
    pub fn new(buffer_factory: &ManagedBufferFactory, capacity: u32) -> Result<Self> {
        let allocator = RangeAllocator::new(capacity);
        let buffer = create_bone_transform_buffer(&buffer_factory, capacity)?;

        Ok(Self { allocator, buffer })
    }

    pub fn allocate(&self, bone_count: u32) -> Allocation {
        self.allocator.allocate(bone_count).unwrap()
    }

    pub fn release(&self, allocation: Allocation) {
        self.allocator.release(allocation);
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        buffer_factory.destroy_buffer(self.buffer.into_managed_buffer())?;

        Ok(())
    }
}
