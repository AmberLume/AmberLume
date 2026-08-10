use render_graph::VirtualBuffer;

#[derive(Copy, Clone)]
pub struct DrawPool {
    pub indirect: VirtualBuffer,
    pub draw_count: VirtualBuffer,
    pub draw_data: VirtualBuffer,
}
