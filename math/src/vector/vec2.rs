use crate::vector::vector_math::vector_math;
use crate::vector::vector_ops::vector_ops;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const ONE: Self = Self::new(1.0, 1.0);

    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub const fn from_array(array: [f32; 2]) -> Self {
        Self::new(array[0], array[1])
    }

    #[inline(always)]
    pub(crate) fn to_glam(self) -> glam::Vec2 {
        glam::Vec2::new(self.x, self.y)
    }

    #[inline(always)]
    pub(crate) fn from_glam(value: glam::Vec2) -> Self {
        Self::new(value.x, value.y)
    }
}

vector_math!(Vec2);
vector_ops!(Vec2, x, y);
