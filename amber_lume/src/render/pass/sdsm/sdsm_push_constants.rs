use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::ids::SliceIndex;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::render::frame_data::sdsm_gpu::SdsmResultGPU;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct SdsmPushConstants {
    pub result_buffer_device_address: DeviceAddress,

    pub depth_descriptor_id: u32,
    pub depth_width: u32,
    pub depth_height: u32,

    pub camera_near: f32,
    pub camera_far: f32,

    _pad0: [u32; 25],
}

impl SdsmPushConstants {
    pub fn create(
        result_buffer: BufferView<SliceBuffer<SdsmResultGPU>>,
        depth_descriptor_id: u32,
        depth_width: u32,
        depth_height: u32,
        camera_near: f32,
        camera_far: f32,
    ) -> Self {
        Self {
            result_buffer_device_address: result_buffer.slice_at(SliceIndex::ZERO).device_address(),

            depth_descriptor_id,
            depth_width,
            depth_height,

            camera_near,
            camera_far,

            _pad0: [0; 25],
        }
    }
}
