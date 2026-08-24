use bytemuck::{Pod, Zeroable};
use std::hash::{Hash, Hasher};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MeshVertexAttributeGPU {
    pub tangent: [f32; 4],
    pub uv: [f32; 2],
}

impl MeshVertexAttributeGPU {
    pub fn new(tangent: [f32; 4], uv: [f32; 2]) -> Self {
        Self { tangent, uv }
    }
}

impl Hash for MeshVertexAttributeGPU {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self { tangent, uv } = self;

        for value in tangent {
            value.to_bits().hash(state);
        }

        for value in uv {
            value.to_bits().hash(state);
        }
    }
}
