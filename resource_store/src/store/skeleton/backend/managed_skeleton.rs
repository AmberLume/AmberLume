use index_allocator::Allocation;

pub struct ManagedSkeleton {
    pub name: String,

    pub bones_allocation: Allocation,
}
