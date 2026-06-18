use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DebugLayerPushConstants {
    pub texture_index: u32,
    pub layer_kind: u32,

    _pad0: [u32; 30],
}

impl DebugLayerPushConstants {
    pub fn create(texture_index: u32, layer_kind: u32) -> Self {
        Self {
            texture_index,
            layer_kind,

            _pad0: [0; 30],
        }
    }
}
