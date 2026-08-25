use ash::vk::DeviceSize;
use crate::resource_state_tracker::buffer_region_key::BufferRegionKey;
use crate::resource_state_tracker::buffer_state::BufferState;

pub struct BufferRegionState {
    pub region: BufferRegionKey,
    pub state: BufferState,
}

impl BufferRegionState {
    pub fn sub_region(
        template: BufferRegionKey,
        state: BufferState,
        offset: DeviceSize,
        end: DeviceSize,
    ) -> Self {
        Self {
            region: BufferRegionKey {
                buffer: template.buffer,
                offset,
                size: end - offset,
            },
            state,
        }
    }
}
