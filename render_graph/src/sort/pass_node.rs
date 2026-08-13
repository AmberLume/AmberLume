use crate::pass_entry::pass_entry::PassEntry;
use crate::virtual_acceleration_structure::virtual_acceleration_structure::VirtualAccelerationStructure;
use crate::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::virtual_data::data_key::DataKey;
use crate::virtual_image::image_subresource::ImageSubresource;

pub struct PassNode {
    pub entry: Box<dyn PassEntry>,
    pub image_reads: Vec<ImageSubresource>,
    pub image_writes: Vec<ImageSubresource>,
    pub buffer_reads: Vec<VirtualBuffer>,
    pub buffer_writes: Vec<VirtualBuffer>,
    pub acceleration_structure_reads: Vec<VirtualAccelerationStructure>,
    pub acceleration_structure_writes: Vec<VirtualAccelerationStructure>,
    pub data_reads: Vec<DataKey>,
}
