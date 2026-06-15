use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DebugLayerPushConstants {
    pub texture_index: u32,
    pub layer_kind: u32,
}
