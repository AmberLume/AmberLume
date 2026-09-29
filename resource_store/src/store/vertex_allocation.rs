use ash::vk::DeviceSize;
use gpu::BufferRange;
use gpu::ManagedBuffer;
use gpu_data::VertexNormalTangentGPU;
use gpu_data::VertexPositionGPU;
use index_allocator::RangeAllocator;

pub struct VertexAllocation {
    pub position: ManagedBuffer,
    pub normal_tangent: ManagedBuffer,
    pub allocator: RangeAllocator,
}

impl VertexAllocation {
    pub fn create(position: ManagedBuffer, normal_tangent: ManagedBuffer, allocator: RangeAllocator) -> Self {
        Self {
            position,
            normal_tangent,
            allocator,
        }
    }

    pub fn position_slice(&self, offset: u32, count: u32) -> BufferRange {
        let item_size = size_of::<VertexPositionGPU>() as DeviceSize;

        self.position.range(offset as DeviceSize * item_size, count as DeviceSize * item_size)
    }

    pub fn normal_tangent_slice(&self, offset: u32, count: u32) -> BufferRange {
        let item_size = size_of::<VertexNormalTangentGPU>() as DeviceSize;

        self.normal_tangent.range(offset as DeviceSize * item_size, count as DeviceSize * item_size)
    }
}
