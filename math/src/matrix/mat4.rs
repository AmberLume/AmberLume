use crate::rotation::quat::Quat;
use crate::vector::vec3::Vec3;
use crate::vector::vec4::Vec4;
use bytemuck::{Pod, Zeroable};
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
    pub const IDENTITY: Self = Self::from_cols(Vec4::X, Vec4::Y, Vec4::Z, Vec4::W);

    #[inline]
    pub const fn from_cols(x_axis: Vec4, y_axis: Vec4, z_axis: Vec4, w_axis: Vec4) -> Self {
        Self {
            x_axis,
            y_axis,
            z_axis,
            w_axis,
        }
    }

    #[inline]
    pub const fn from_cols_array_2d(array: &[[f32; 4]; 4]) -> Self {
        Self::from_cols(
            Vec4::from_array(array[0]),
            Vec4::from_array(array[1]),
            Vec4::from_array(array[2]),
            Vec4::from_array(array[3]),
        )
    }

    #[inline]
    pub const fn to_cols_array_2d(self) -> [[f32; 4]; 4] {
        [
            self.x_axis.to_array(),
            self.y_axis.to_array(),
            self.z_axis.to_array(),
            self.w_axis.to_array(),
        ]
    }

    #[inline]
    pub fn from_translation(translation: Vec3) -> Self {
        Self::from_glam(glam::Mat4::from_translation(translation.glam()))
    }

    #[inline]
    pub fn from_scale_rotation_translation(scale: Vec3, rotation: Quat, translation: Vec3) -> Self {
        Self::from_glam(glam::Mat4::from_scale_rotation_translation(
            scale.glam(),
            rotation.glam(),
            translation.glam(),
        ))
    }

    #[inline]
    pub fn perspective_rh(fov_y_radians: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
        Self::from_glam(glam::camera::rh::proj::directx::perspective(fov_y_radians, aspect_ratio, z_near, z_far))
    }

    #[inline]
    pub fn look_at_rh(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        Self::from_glam(glam::camera::rh::view::look_at_mat4(eye.glam(), center.glam(), up.glam()))
    }

    #[inline]
    pub fn inverse(self) -> Self {
        Self::from_glam(self.glam().inverse())
    }

    #[inline]
    pub fn transpose(self) -> Self {
        Self::from_glam(self.glam().transpose())
    }

    #[inline]
    pub fn row(self, index: usize) -> Vec4 {
        Vec4::from_glam(self.glam().row(index))
    }

    #[inline]
    pub fn transform_point3(self, point: Vec3) -> Vec3 {
        Vec3::from_glam(self.glam().transform_point3(point.glam()))
    }

    #[inline]
    pub fn transform_vector3(self, vector: Vec3) -> Vec3 {
        Vec3::from_glam(self.glam().transform_vector3(vector.glam()))
    }

    #[inline]
    pub(crate) fn glam(self) -> glam::Mat4 {
        glam::Mat4::from_cols(
            self.x_axis.glam(),
            self.y_axis.glam(),
            self.z_axis.glam(),
            self.w_axis.glam(),
        )
    }

    #[inline]
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

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self::from_glam(self.glam() * rhs.glam())
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    #[inline]
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4::from_glam(self.glam() * rhs.glam())
    }
}
