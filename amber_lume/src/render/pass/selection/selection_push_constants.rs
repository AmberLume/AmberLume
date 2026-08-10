use bytemuck::{Pod, Zeroable};

#[repr(C, align(4))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SelectionPushConstants {
    pub color: [f32; 4],

    pub entity_id_texel_scale: [f32; 2],

    pub entity_id_texture: u32,
    pub selected_entity: u32,

    pub stripe_width: f32,

    _pad0: [u32; 23],
}

impl SelectionPushConstants {
    pub fn create(
        color: [f32; 4],
        entity_id_texel_scale: [f32; 2],
        entity_id_texture: u32,
        selected_entity: u32,
        stripe_width: f32,
    ) -> Self {
        Self {
            color,
            entity_id_texel_scale,
            entity_id_texture,
            selected_entity,
            stripe_width,

            _pad0: [0; 23],
        }
    }
}
