use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;

#[derive(Copy, Clone)]
pub struct DrawPool {
    pub indirect: VirtualBuffer,
    pub draw_count: VirtualBuffer,
    pub draw_data: VirtualBuffer,
}
