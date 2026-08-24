use bytemuck::{Pod, Zeroable};
use std::hash::{Hash, Hasher};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MeshVertexGPU {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl MeshVertexGPU {
    pub fn new(position: [f32; 3], normal: [f32; 3]) -> Self {
        Self {
            position,
            normal,
        }
    }
}

impl Hash for MeshVertexGPU {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self { position, normal } = self;

        for v in position {
            v.to_bits().hash(state);
        }

        for v in normal {
            v.to_bits().hash(state);
        }
    }
}
