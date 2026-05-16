use crate::render::render_graph::resource_state_tracker::buffer_region_key::BufferRegionKey;
use crate::render::render_graph::resource_state_tracker::buffer_state::BufferState;

pub struct BufferRegionState {
    pub region: BufferRegionKey,
    pub state: BufferState,
}
