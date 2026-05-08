use crate::render::render_graph::pass_entry::pass_entry::PassEntry;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;

pub struct PassNode {
    pub entry: Box<dyn PassEntry>,
    pub image_reads: Vec<VirtualImage>,
    pub image_writes: Vec<VirtualImage>,
    pub buffer_reads: Vec<VirtualBuffer>,
    pub buffer_writes: Vec<VirtualBuffer>,
}
