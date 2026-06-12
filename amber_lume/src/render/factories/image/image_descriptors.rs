use crate::resources::bindless::bindless_image_array::BindlessImageArray;

pub struct ImageDescriptors {
    pub storage_mips: Option<BindlessImageArray>,
}
