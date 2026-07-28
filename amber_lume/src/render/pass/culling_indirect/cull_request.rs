use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;

#[derive(Copy, Clone)]
pub struct CullRequest {
    pub accept_mask: u32,

    pub draw_count: VirtualBuffer,
    pub indirect: VirtualBuffer,
    pub draw_data: VirtualBuffer,
}
