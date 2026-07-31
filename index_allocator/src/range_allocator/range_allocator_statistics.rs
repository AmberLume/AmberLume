use crate::range_allocator::range_allocator::Allocation;

pub struct RangeAllocatorStatistics {
    pub capacity: u32,
    pub used: u32,
    pub free: u32,

    pub free_blocks: Vec<Allocation>,

    pub largest_free: u32,
    pub fragmentation: u32,
}
