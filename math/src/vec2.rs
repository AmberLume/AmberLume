use crate::vector_math::vector_math;
use crate::vector_ops::vector_ops;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self::splat(0.0);
    pub const ONE: Self = Self::splat(1.0);
    pub const X: Self = Self::new(1.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0);

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn splat(value: f32) -> Self {
        Self::new(value, value)
    }

    #[inline]
    pub const fn from_array(array: [f32; 2]) -> Self {
        Self::new(array[0], array[1])
    }

    #[inline]
    pub const fn to_array(self) -> [f32; 2] {
        [self.x, self.y]
    }

    #[inline]
    pub(crate) fn glam(self) -> glam::Vec2 {
        glam::Vec2::new(self.x, self.y)
    }

    #[inline]
    pub(crate) fn from_glam(value: glam::Vec2) -> Self {
        Self::new(value.x, value.y)
    }
}

vector_math!(Vec2);
vector_ops!(Vec2, x, y);
