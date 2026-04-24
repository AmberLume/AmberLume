use std::hash::{Hash, Hasher};
use anyhow::Result;
use ash::vk;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use vk::BufferUsageFlags;
use crate::data::submesh_data::ArchivedSubmeshData;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct VertexGPU {
    pub position: [f32; 3],
    pub _pad0: f32,
    pub normal: [f32; 3],
    pub _pad1: f32,
    pub tangent: [f32; 4],
    pub uv: [f32; 2],
    pub bone_indices: [u32; 2],
    pub bone_weights: [f32; 4],
}

impl VertexGPU {
    pub fn new(
        position: [f32; 3],
        normal: [f32; 3],
        tangent: [f32; 4],
        uv: [f32; 2],
        bone_indices: [u32; 2],
        bone_weights: [f32; 4],
    ) -> Self {
        Self {
            position,
            _pad0: 0.0,
            normal,
            _pad1: 0.0,
            tangent,
            uv,
            bone_indices,
            bone_weights,
        }
    }

    pub fn from(submesh_data: &ArchivedSubmeshData, index: usize) -> Self {
        let position = &submesh_data.positions[index];
        let normal = &submesh_data.normals[index];
        let tangent = &submesh_data.tangents[index];
        let uv = &submesh_data.uvs[index];
        let bone_indices = &submesh_data.bone_indices[index];
        let bone_weights = &submesh_data.bone_weights[index];

        Self::new(
            position.map(|v| v.into()),
            normal.map(|v| v.into()),
            tangent.map(|v| v.into()),
            uv.map(|v| v.into()),
            [
                (bone_indices[0].to_native() as u32) | ((bone_indices[1].to_native() as u32) << 16),
                (bone_indices[2].to_native() as u32) | ((bone_indices[3].to_native() as u32) << 16),
            ],
            bone_weights.map(|v| v.into()),
        )
    }
}

impl Hash for VertexGPU {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self {
            position,
            _pad0: _,
            normal,
            _pad1: _,
            tangent,
            uv,
            bone_indices,
            bone_weights,
        } = self;

        for v in position {
            v.to_bits().hash(state);
        }
        for v in normal {
            v.to_bits().hash(state);
        }
        for v in tangent {
            v.to_bits().hash(state);
        }
        for v in uv {
            v.to_bits().hash(state);
        }
        for v in bone_indices {
            v.hash(state);
        }
        for v in bone_weights {
            v.to_bits().hash(state);
        }
    }
}

pub fn create_vertex_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<VertexGPU>> {
    BufferBuilder::slice(capacity)
        .build(
            buffer_factory,
            "vertex",
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
        )
}
