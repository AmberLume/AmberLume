use crate::rotation::euler_order::EulerOrder;
use crate::vector::vec3::Vec3;
use bytemuck::{Pod, Zeroable};
use glam::EulerRot;
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

    #[inline(always)]
    pub const fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline(always)]
    pub const fn from_array(array: [f32; 4]) -> Self {
        Self::from_xyzw(array[0], array[1], array[2], array[3])
    }

    #[inline(always)]
    pub fn from_euler(order: EulerOrder, x: f32, y: f32, z: f32) -> Self {
        let rotation = match order {
            EulerOrder::Xyz => glam::Quat::from_euler(EulerRot::XYZ, x, y, z),
            EulerOrder::Yxz => glam::Quat::from_euler(EulerRot::YXZ, y, x, z),
        };

        Self::from_glam(rotation)
    }

    #[inline(always)]
    pub fn to_euler(self, order: EulerOrder) -> (f32, f32, f32) {
        match order {
            EulerOrder::Xyz => self.to_glam().to_euler(EulerRot::XYZ),
            EulerOrder::Yxz => {
                let (y, x, z) = self.to_glam().to_euler(EulerRot::YXZ);

                (x, y, z)
            }
        }
    }

    #[inline(always)]
    pub fn to_scaled_axis(self) -> Vec3 {
        Vec3::from_glam(self.to_glam().to_scaled_axis())
    }

    #[inline(always)]
    pub fn dot(self, rhs: Self) -> f32 {
        self.to_glam().dot(rhs.to_glam())
    }

    #[inline(always)]
    pub fn inverse(self) -> Self {
        Self::from_glam(self.to_glam().inverse())
    }

    #[inline(always)]
    pub fn normalize(self) -> Self {
        Self::from_glam(self.to_glam().normalize())
    }

    #[inline(always)]
    pub fn slerp(self, end: Self, t: f32) -> Self {
        Self::from_glam(self.to_glam().slerp(end.to_glam(), t))
    }

    #[inline(always)]
    pub(crate) fn to_glam(self) -> glam::Quat {
        glam::Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }

    #[inline(always)]
    pub(crate) fn from_glam(value: glam::Quat) -> Self {
        Self::from_xyzw(value.x, value.y, value.z, value.w)
    }
}

impl Mul for Quat {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        Self::from_glam(self.to_glam() * rhs.to_glam())
    }
}

impl Mul<Vec3> for Quat {
    type Output = Vec3;

    #[inline(always)]
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::from_glam(self.to_glam() * rhs.to_glam())
    }
}

impl Neg for Quat {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        Self::from_xyzw(-self.x, -self.y, -self.z, -self.w)
    }
}
