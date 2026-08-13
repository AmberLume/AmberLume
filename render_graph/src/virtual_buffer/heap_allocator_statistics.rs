#[derive(Default, Clone, Copy)]
pub struct HeapAllocatorStatistics {
    pub capacity: u32,
    pub used: u32,
}
