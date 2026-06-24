use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DebugLayerPushConstants {
    pub inverse_view_projection: [[f32; 4]; 4],

    pub texture_index: u32,
    pub layer_kind: u32,

    _pad0: [u32; 14],
}

impl DebugLayerPushConstants {
    pub fn create(inverse_view_projection: [[f32; 4]; 4], texture_index: u32, layer_kind: u32) -> Self {
        Self {
            inverse_view_projection,

            texture_index,
            layer_kind,

            _pad0: [0; 14],
        }
    }
}
