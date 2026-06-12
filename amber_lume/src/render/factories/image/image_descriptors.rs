use crate::resources::bindless::bindless_image::BindlessImage;
use crate::resources::bindless::bindless_image_array::BindlessImageArray;

pub struct ImageDescriptors {
    pub view: Option<BindlessImage>,
    pub storage_mips: Option<BindlessImageArray>,
}
