use crate::limits::ResourceLimits;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::resources::range_allocator::range_allocator::{Allocation, RangeAllocator};
use crate::resources::skinning::skinning_buffer::{
    create_skinning_instance_buffer, SkinningInstanceGPU,
};
use anyhow::Result;

pub struct BoneTransformHandler {
    allocator: RangeAllocator,

    pub(crate) skinning_instance_buffer: SliceBuffer<SkinningInstanceGPU>,
}

impl BoneTransformHandler {
    pub fn new(buffer_factory: &ManagedBufferFactory, limits: &ResourceLimits) -> Result<Self> {
        let allocator = RangeAllocator::new(limits.max_bone_transforms);

        let skinning_instance_buffer = create_skinning_instance_buffer(&buffer_factory, limits.max_skinning_instances)?;

        Ok(Self {
            allocator,
            skinning_instance_buffer,
        })
    }

    pub fn allocate(&self, bone_count: u32) -> Allocation {
        self.allocator.allocate(bone_count).unwrap()
    }

    pub fn release(&self, allocation: Allocation) {
        self.allocator.release(allocation);
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        buffer_factory.destroy_buffer(self.skinning_instance_buffer.into_managed_buffer())?;

        Ok(())
    }
}
