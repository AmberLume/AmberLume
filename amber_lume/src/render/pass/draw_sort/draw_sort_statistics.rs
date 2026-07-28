use bytemuck::{Pod, Zeroable};

pub const DRAW_SORT_META_NAME: &str = "draw_sort.transparent";

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct DrawSortStatisticsGPU {
    pub sorted_count: u32,
    pub unsorted_count: u32,

    pub _pad0: [u32; 2],
}
