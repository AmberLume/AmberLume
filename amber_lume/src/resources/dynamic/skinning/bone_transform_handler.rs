use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::resources::dynamic::skinning::bone_transform_buffer::{
    BoneTransformGPU, create_bone_transform_buffer,
};
use crate::resources::range_allocator::range_allocator::{Allocation, RangeAllocator};
use anyhow::Result;
use crate::limits::renderer_limits::RendererLimits;
use crate::resources::dynamic::skinning::skinning_buffer::{create_skinning_instance_buffer, SkinningInstanceGPU};

pub struct BoneTransformHandler {
    allocator: RangeAllocator,
    
    pub(crate) bone_transform_buffer: SliceBuffer<BoneTransformGPU>,
    pub(crate) skinning_instance_buffer: SliceBuffer<SkinningInstanceGPU>,
}

impl BoneTransformHandler {
    pub fn new(
        buffer_factory: &ManagedBufferFactory,
        render_limits: &RendererLimits,
    ) -> Result<Self> {
        let allocator = RangeAllocator::new(render_limits.render_resource_limits.max_bone_transforms);
        
        let bone_transform_buffer = create_bone_transform_buffer(
            &buffer_factory, 
            render_limits.render_resource_limits.max_bone_transforms,
        )?;
        let skinning_instance_buffer = create_skinning_instance_buffer(
            &buffer_factory,
            render_limits.render_resource_limits.max_skinning_instances,
        )?;

        Ok(Self { 
            allocator,
            bone_transform_buffer,
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
        buffer_factory.destroy_buffer(self.bone_transform_buffer.into_managed_buffer())?;
        buffer_factory.destroy_buffer(self.skinning_instance_buffer.into_managed_buffer())?;

        Ok(())
    }
}
