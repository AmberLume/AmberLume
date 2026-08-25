#[derive(Default, Clone, Copy)]
pub struct BlockHeapStatistics {
    pub block_size: u32,
    pub block_count: u32,
    pub oversize_count: u32,
    pub used: u32,
    pub peak_used: u32,
    pub capacity: u32,
}
