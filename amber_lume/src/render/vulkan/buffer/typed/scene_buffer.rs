use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::vulkan::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::vulkan::factories::buffer::typed_buffer::typed_buffer::TypedBuffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MainCameraGpuData {
    pub projection_matrix: [[f32; 4]; 4],

    pub position: [f32; 3],
    _pad0: u32,

    pub near: f32,
    pub far: f32,
    _pad1: [u32; 2],
}

impl MainCameraGpuData {
    pub fn new(
        projection_matrix: [[f32; 4]; 4],
        position: [f32; 3],
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            projection_matrix,

            position,
            _pad0: 0,

            near,
            far,
            _pad1: [0; 2],
        }
    }
}

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct ShadowCascadeGpuData {
    pub light_space_matrix: [[f32; 4]; 4],
    pub screen_to_light: [[f32; 4]; 4],
    pub split: f32,
    _pad0: [u32; 3],
}

impl ShadowCascadeGpuData {
    pub fn new(
        light_space_matrix: [[f32; 4]; 4],
        screen_to_light: [[f32; 4]; 4],
        split: f32,
    ) -> Self {
        Self {
            light_space_matrix,
            screen_to_light,
            split,
            _pad0: [0; 3],
        }
    }
}

impl Default for ShadowCascadeGpuData {
    fn default() -> Self {
        Self {
            light_space_matrix: [[0.0; 4]; 4],
            screen_to_light: [[0.0; 4]; 4],
            split: 0.0,

            _pad0: [0; 3],
        }
    }
}

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SceneGpuData {
    pub main_camera: MainCameraGpuData,

    pub light_direction: [f32; 3],
    _pad0: u32,

    pub cascades: [ShadowCascadeGpuData; 4],
    pub cascade_count: u32,

    _pad1: [u32; 3],
}

impl SceneGpuData {
    pub fn create(
        main_camera: MainCameraGpuData,
        light_direction: [f32; 3],
        cascade_count: u32,
        cascades: [ShadowCascadeGpuData; 4],
    ) -> Self {
        Self {
            main_camera,

            light_direction,
            _pad0: 0,

            cascades,
            cascade_count,

            _pad1: [0; 3],
        }
    }
}

pub fn create_scene_buffer(
    buffer_factory: &ManagedBufferFactory,
    frame_count: u32,
) -> Result<FrameBuffer<TypedBuffer<SceneGpuData>>> {
    BufferBuilder::typed()
        .per_frame(frame_count)
        .build(
            buffer_factory,
            "scene_buffer",
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
        )
}
