use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SelectionPushConstants {
    pub color: [f32; 4],

    pub entity_id_texel_scale: [f32; 2],

    pub entity_id_texture: u32,
    pub selected_entity: u32,

    pub stripe_width: f32,
}
