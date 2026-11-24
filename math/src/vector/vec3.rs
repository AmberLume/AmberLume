use crate::vector::vec4::Vec4;
use crate::vector::vector_math::vector_math;
use crate::vector::vector_ops::vector_ops;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self::splat(0.0);
    pub const ONE: Self = Self::splat(1.0);
    pub const X: Self = Self::new(1.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);
    pub const NEG_X: Self = Self::new(-1.0, 0.0, 0.0);
    pub const NEG_Y: Self = Self::new(0.0, -1.0, 0.0);
    pub const NEG_Z: Self = Self::new(0.0, 0.0, -1.0);

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn splat(value: f32) -> Self {
        Self::new(value, value, value)
    }

    #[inline]
    pub const fn from_array(array: [f32; 3]) -> Self {
        Self::new(array[0], array[1], array[2])
    }

    #[inline]
    pub const fn to_array(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    #[inline]
    pub const fn extend(self, w: f32) -> Vec4 {
        Vec4::new(self.x, self.y, self.z, w)
    }

    #[inline]
    pub fn cross(self, rhs: Self) -> Self {
        Self::from_glam(self.glam().cross(rhs.glam()))
    }

    #[inline]
    pub fn any_orthonormal_vector(self) -> Self {
        Self::from_glam(self.glam().any_orthonormal_vector())
    }

    #[inline]
    pub(crate) fn glam(self) -> glam::Vec3 {
        glam::Vec3::new(self.x, self.y, self.z)
    }

    #[inline]
    pub(crate) fn from_glam(value: glam::Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

vector_math!(Vec3);
vector_ops!(Vec3, x, y, z);
