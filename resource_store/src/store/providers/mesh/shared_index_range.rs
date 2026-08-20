use index_allocator::Allocation;

pub struct SharedIndexRange {
    pub allocation: Allocation,
    pub users: usize,
}
