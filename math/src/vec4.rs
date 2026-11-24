use crate::vec3::Vec3;
use crate::vector_math::vector_math;
use crate::vector_ops::vector_ops;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const ZERO: Self = Self::splat(0.0);
    pub const ONE: Self = Self::splat(1.0);
    pub const X: Self = Self::new(1.0, 0.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0, 0.0);
    pub const W: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub const fn splat(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    #[inline]
    pub const fn from_array(array: [f32; 4]) -> Self {
        Self::new(array[0], array[1], array[2], array[3])
    }

    #[inline]
    pub const fn to_array(self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }

    #[inline]
    pub const fn truncate(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    #[inline]
    pub(crate) fn glam(self) -> glam::Vec4 {
        glam::Vec4::new(self.x, self.y, self.z, self.w)
    }

    #[inline]
    pub(crate) fn from_glam(value: glam::Vec4) -> Self {
        Self::new(value.x, value.y, value.z, value.w)
    }
}

vector_math!(Vec4);
vector_ops!(Vec4, x, y, z, w);
