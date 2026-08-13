use std::hash::{Hash, Hasher};
use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SkeletonBoneGPU {
    pub parent: i32,

    _pad0: [u32; 3],

    pub inverse_bind_matrix: [[f32; 4]; 4],
}

impl SkeletonBoneGPU {
    pub fn create(parent: i32, inverse_bind_matrix: [[f32; 4]; 4]) -> Self {
        Self {
            parent,

            _pad0: [0; 3],

            inverse_bind_matrix,
        }
    }
}

impl Hash for SkeletonBoneGPU {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self {
            parent,

            _pad0: _,

            inverse_bind_matrix,
        } = self;

        parent.hash(state);

        for row in inverse_bind_matrix {
            for value in row {
                value.to_bits().hash(state);
            }
        }
    }
}
