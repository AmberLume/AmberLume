use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct DepthPyramidPushConstants {
    pub depth_descriptor_id: u32,
    pub width: u32,
    pub height: u32,
    pub mip_count: u32,

    pub mip0_id: u32,
    pub mip1_id: u32,
    pub mip2_id: u32,
    pub mip3_id: u32,
    pub mip4_id: u32,

    _pad0: [u32; 23],
}

impl DepthPyramidPushConstants {
    pub fn create(
        depth_descriptor_id: u32,
        width: u32,
        height: u32,
        mip_ids: [u32; 5],
    ) -> Self {
        Self {
            depth_descriptor_id,
            width,
            height,
            mip_count: mip_ids.len() as u32,

            mip0_id: mip_ids[0],
            mip1_id: mip_ids[1],
            mip2_id: mip_ids[2],
            mip3_id: mip_ids[3],
            mip4_id: mip_ids[4],

            _pad0: [0; 23],
        }
    }
}
