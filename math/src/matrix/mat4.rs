use crate::rotation::quat::Quat;
use crate::vector::vec3::Vec3;
use crate::vector::vec4::Vec4;
use bytemuck::{Pod, Zeroable};
use glam::camera::rh::proj::directx::perspective;
use glam::camera::rh::view::look_at_mat4;
use std::ops::Mul;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Mat4 {
    pub x_axis: Vec4,
    pub y_axis: Vec4,
    pub z_axis: Vec4,
    pub w_axis: Vec4,
}

impl Mat4 {
    pub const ZERO: Self = Self::from_cols(Vec4::ZERO, Vec4::ZERO, Vec4::ZERO, Vec4::ZERO);

    #[inline(always)]
    pub const fn from_cols(x_axis: Vec4, y_axis: Vec4, z_axis: Vec4, w_axis: Vec4) -> Self {
        Self {
            x_axis,
            y_axis,
            z_axis,
            w_axis,
        }
    }

    #[inline(always)]
    pub const fn from_cols_array_2d(array: &[[f32; 4]; 4]) -> Self {
        Self::from_cols(
            Vec4::from_array(array[0]),
            Vec4::from_array(array[1]),
            Vec4::from_array(array[2]),
            Vec4::from_array(array[3]),
        )
    }

    #[inline(always)]
    pub fn from_translation(translation: Vec3) -> Self {
        Self::from_glam(glam::Mat4::from_translation(translation.to_glam()))
    }

    #[inline(always)]
    pub fn from_scale_rotation_translation(scale: Vec3, rotation: Quat, translation: Vec3) -> Self {
        Self::from_glam(glam::Mat4::from_scale_rotation_translation(
            scale.to_glam(),
            rotation.to_glam(),
            translation.to_glam(),
        ))
    }

    #[inline(always)]
    pub fn perspective_rh(fov_y_radians: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
        Self::from_glam(perspective(fov_y_radians, aspect_ratio, z_near, z_far))
    }

    #[inline(always)]
    pub fn look_at_rh(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        Self::from_glam(look_at_mat4(eye.to_glam(), center.to_glam(), up.to_glam()))
    }

    #[inline(always)]
    pub fn inverse(self) -> Self {
        Self::from_glam(self.to_glam().inverse())
    }

    #[inline(always)]
    pub fn row(self, index: usize) -> Vec4 {
        Vec4::from_glam(self.to_glam().row(index))
    }

    #[inline(always)]
    pub fn transform_point3(self, point: Vec3) -> Vec3 {
        Vec3::from_glam(self.to_glam().transform_point3(point.to_glam()))
    }

    #[inline(always)]
    pub(crate) fn to_glam(self) -> glam::Mat4 {
        glam::Mat4::from_cols(
            self.x_axis.to_glam(),
            self.y_axis.to_glam(),
            self.z_axis.to_glam(),
            self.w_axis.to_glam(),
        )
    }

    #[inline(always)]
    pub(crate) fn from_glam(value: glam::Mat4) -> Self {
        Self::from_cols(
            Vec4::from_glam(value.x_axis),
            Vec4::from_glam(value.y_axis),
            Vec4::from_glam(value.z_axis),
            Vec4::from_glam(value.w_axis),
        )
    }
}

impl Mul for Mat4 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        Self::from_glam(self.to_glam() * rhs.to_glam())
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    #[inline(always)]
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4::from_glam(self.to_glam() * rhs.to_glam())
    }
}
