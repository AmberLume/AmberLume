use bytemuck::{Pod, Zeroable};
use render_graph::DrawBucket;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct CullRequestGPU {
    pub accept_mask: u32,
    pub count_index: u32,
    pub draw_offset: u32,
    pub capacity: u32,
}

impl CullRequestGPU {
    pub fn create(accept_mask: u32, bucket: DrawBucket) -> Self {
        Self {
            accept_mask,
            count_index: bucket.count_index,
            draw_offset: bucket.draw_offset,
            capacity: bucket.capacity,
        }
    }
}
