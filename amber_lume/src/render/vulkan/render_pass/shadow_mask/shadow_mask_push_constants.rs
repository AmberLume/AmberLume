use bytemuck::{Pod, Zeroable};
use crate::resources::dynamic::resource_provider::ResourceId;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShadowMaskPushConstants {
    pub screen_to_light: [[f32; 4]; 4],

    pub depth_descriptor_id: u32,
    pub global_shadow_descriptor_id: u32,
}

impl ShadowMaskPushConstants {
    pub fn create(
        screen_to_light: [[f32; 4]; 4],
        depth_descriptor_id: ResourceId,
        global_shadow_descriptor_id: ResourceId,
    ) -> Self {
        Self {
            screen_to_light,

            depth_descriptor_id,
            global_shadow_descriptor_id,
        }
    }
}
