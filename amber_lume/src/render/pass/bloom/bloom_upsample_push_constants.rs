use bytemuck::{Pod, Zeroable};

#[repr(C, align(4))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct BloomUpsamplePushConstants {
    pub source_descriptor_id: u32,

    _pad0: [u32; 31],
}

impl BloomUpsamplePushConstants {
    pub fn create(source_descriptor_id: u32) -> Self {
        Self {
            source_descriptor_id,

            _pad0: [0; 31],
        }
    }
}
