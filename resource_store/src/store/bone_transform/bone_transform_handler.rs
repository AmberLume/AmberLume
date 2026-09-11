use index_allocator::ResourceLimits;
use index_allocator::Allocation;
use index_allocator::RangeAllocator;

pub struct BoneTransformHandler {
    allocator: RangeAllocator,
}

impl BoneTransformHandler {
    pub fn new(limits: &ResourceLimits) -> Self {
        Self {
            allocator: RangeAllocator::new(limits.max_bone_transforms),
        }
    }

    pub fn allocate(&self, bone_count: u32) -> Allocation {
        self.allocator.allocate(bone_count).unwrap()
    }

    pub fn release(&self, allocation: Allocation) {
        self.allocator.release(allocation);
    }
}
