use bytemuck::{Pod, Zeroable};
use std::hash::{Hash, Hasher};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MeshVertexSkinGPU {
    pub bone_indices: [u32; 2],
    pub bone_weights: [f32; 4],
}

impl MeshVertexSkinGPU {
    pub fn new(bone_indices: [u32; 2], bone_weights: [f32; 4]) -> Self {
        Self {
            bone_indices,
            bone_weights,
        }
    }
}

impl Hash for MeshVertexSkinGPU {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self { bone_indices, bone_weights } = self;

        bone_indices.hash(state);

        for value in bone_weights {
            value.to_bits().hash(state);
        }
    }
}
