use crate::vector::vec3::Vec3;
use crate::vector::vector_math::vector_math;
use crate::vector::vector_ops::vector_ops;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const W: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline(always)]
    pub const fn from_array(array: [f32; 4]) -> Self {
        Self::new(array[0], array[1], array[2], array[3])
    }

    #[inline(always)]
    pub const fn to_array(self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }

    #[inline(always)]
    pub const fn truncate(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    #[inline(always)]
    pub(crate) fn to_glam(self) -> glam::Vec4 {
        glam::Vec4::new(self.x, self.y, self.z, self.w)
    }

    #[inline(always)]
    pub(crate) fn from_glam(value: glam::Vec4) -> Self {
        Self::new(value.x, value.y, value.z, value.w)
    }
}

vector_math!(Vec4);
vector_ops!(Vec4, x, y, z, w);
