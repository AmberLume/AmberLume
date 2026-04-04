use std::hash::{Hash, Hasher};
use ash::vk::{ImageAspectFlags, ImageViewType};

#[derive(Copy, Clone, Debug)]
pub struct ImageViewDescription {
    pub image_view_type: ImageViewType,
    pub image_aspect_flags: ImageAspectFlags,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,

    pub layered: bool,
}

impl ImageViewDescription {
    pub fn default_2d_color() -> Self {
        Self {
            image_view_type: ImageViewType::TYPE_2D,
            image_aspect_flags: ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,

            layered: false,
        }
    }

    pub fn default_2d_array_depth(layer_count: u32) -> Self {
        Self {
            image_view_type: ImageViewType::TYPE_2D_ARRAY,
            image_aspect_flags: ImageAspectFlags::DEPTH,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count,

            layered: true,
        }
    }
}

impl Hash for ImageViewDescription {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self {
            image_view_type,
            image_aspect_flags,
            base_mip_level,
            level_count,
            base_array_layer,
            layer_count,

            layered,
        } = self;
        
        image_view_type.as_raw().hash(state);
        image_aspect_flags.hash(state);
        base_mip_level.hash(state);
        level_count.hash(state);
        base_array_layer.hash(state);
        layer_count.hash(state);
        
        layered.hash(state);
    }
}
