use std::hash::{Hash, Hasher};
use bytemuck::{Pod, Zeroable};

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
