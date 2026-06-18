use crate::render::render_graph::resource_state_tracker::image_region_key::ImageRegionKey;
use crate::render::render_graph::resource_state_tracker::image_state::ImageState;

pub struct ImageRegionState {
    pub region: ImageRegionKey,
    pub state: ImageState,
}
