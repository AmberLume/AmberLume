use crate::render::render_graph::resource_state_tracker::image_region_key::ImageRegionKey;
use crate::render::render_graph::resource_state_tracker::image_state::ImageState;

pub struct ImageRegionState {
    pub region: ImageRegionKey,
    pub state: ImageState,
}

impl ImageRegionState {
    pub fn sub_region(
        template: ImageRegionKey,
        state: ImageState,
        base_mip_level: u32,
        mip_end: u32,
        base_array_layer: u32,
        layer_end: u32,
    ) -> Self {
        Self {
            region: ImageRegionKey {
                image: template.image,
                aspect_mask: template.aspect_mask,
                base_mip_level,
                level_count: mip_end - base_mip_level,
                base_array_layer,
                layer_count: layer_end - base_array_layer,
            },
            state,
        }
    }
}
