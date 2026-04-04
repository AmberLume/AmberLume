use crate::render::render_graph::pass_entry::pass_entry::PassEntry;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;

pub struct PassNode {
    pub entry: Box<dyn PassEntry>,
    pub reads: Vec<VirtualImage>,
    pub writes: Vec<VirtualImage>,
}
