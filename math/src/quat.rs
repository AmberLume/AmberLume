use crate::euler_order::EulerOrder;
use crate::vec3::Vec3;
use bytemuck::{Pod, Zeroable};
use std::ops::{Mul, Neg};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Self = Self::from_xyzw(0.0, 0.0, 0.0, 1.0);

    #[inline]
    pub const fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub const fn from_array(array: [f32; 4]) -> Self {
        Self::from_xyzw(array[0], array[1], array[2], array[3])
    }

    #[inline]
    pub const fn to_array(self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }

    #[inline]
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        Self::from_glam(glam::Quat::from_axis_angle(axis.glam(), angle))
    }

    #[inline]
    pub fn from_scaled_axis(scaled_axis: Vec3) -> Self {
        Self::from_glam(glam::Quat::from_scaled_axis(scaled_axis.glam()))
    }

    #[inline]
    pub fn to_scaled_axis(self) -> Vec3 {
        Vec3::from_glam(self.glam().to_scaled_axis())
    }

    #[inline]
    pub fn from_euler(order: EulerOrder, a: f32, b: f32, c: f32) -> Self {
        Self::from_glam(glam::Quat::from_euler(order.glam(), a, b, c))
    }

    #[inline]
    pub fn to_euler(self, order: EulerOrder) -> (f32, f32, f32) {
        self.glam().to_euler(order.glam())
    }

    #[inline]
    pub fn dot(self, rhs: Self) -> f32 {
        self.glam().dot(rhs.glam())
    }

    #[inline]
    pub fn inverse(self) -> Self {
        Self::from_glam(self.glam().inverse())
    }

    #[inline]
    pub fn normalize(self) -> Self {
        Self::from_glam(self.glam().normalize())
    }

    #[inline]
    pub fn slerp(self, end: Self, t: f32) -> Self {
        Self::from_glam(self.glam().slerp(end.glam(), t))
    }

    #[inline]
    pub(crate) fn glam(self) -> glam::Quat {
        glam::Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }

    #[inline]
    pub(crate) fn from_glam(value: glam::Quat) -> Self {
        Self::from_xyzw(value.x, value.y, value.z, value.w)
    }
}

impl Mul for Quat {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self::from_glam(self.glam() * rhs.glam())
    }
}

impl Mul<Vec3> for Quat {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::from_glam(self.glam() * rhs.glam())
    }
}

impl Neg for Quat {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self::from_xyzw(-self.x, -self.y, -self.z, -self.w)
    }
}
