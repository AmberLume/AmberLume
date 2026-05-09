use std::ops::Range;
use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};
use crate::utils::matrix_wrappers::{ProjectionMatrix, ViewMatrix, ViewProjectionMatrix};

pub struct ShadowCascadeHelper;

impl ShadowCascadeHelper {
    pub fn compute_cascade_ranges(
        z_min: f32,
        z_max: f32,
        cascade_count: usize,
    ) -> Vec<Range<f32>> {
        let z_min = z_min.max(1e-3);
        if cascade_count == 0 || z_min >= z_max {
            return vec![z_min..z_max.max(z_min + 1.0)];
        }

        const LAMBDA: f32 = 0.7;
        let n = cascade_count as f32;
        let ratio = z_max / z_min;
        let span = z_max - z_min;

        let mut splits = Vec::with_capacity(cascade_count + 1);
        splits.push(z_min);
        for i in 1..cascade_count {
            let t = i as f32 / n;
            let c_log = z_min * ratio.powf(t);
            let c_uniform = z_min + span * t;
            splits.push(LAMBDA * c_log + (1.0 - LAMBDA) * c_uniform);
        }
        splits.push(z_max);

        (0..cascade_count).map(|i| splits[i]..splits[i + 1]).collect()
    }

    pub fn from_camera_projection(
        camera_view: &ViewMatrix,
        camera_projection: &ProjectionMatrix,
        shadow_cascades: &[Range<f32>],
        shadow_map_resolution: u32,
        light_direction: Vec3,
        camera_near: f32,
        camera_far: f32,
        light_margin: f32,
    ) -> Vec<ViewProjectionMatrix> {
        let view_projection = ViewProjectionMatrix::from_view_projection(camera_view, &camera_projection);
        let original_corners = Self::get_frustum_corners(&view_projection);
        let light_direction = light_direction.normalize();

        shadow_cascades.iter().map(|cascade_range| {
            let cascade_corners = Self::get_cascade_corners(
                &original_corners,
                cascade_range,
                camera_near,
                camera_far,
            );
            let cascade_center = Self::get_frustum_center(&cascade_corners);
            let cascade_radius = Self::get_frustum_radius(&cascade_corners, cascade_center);

            let light_up = if light_direction.dot(Vec3::Y).abs() > 0.99 { Vec3::Z } else { Vec3::Y };

            let light_position = cascade_center - light_direction * (cascade_radius + light_margin);
            let light_view = ViewMatrix::from_mat4(Mat4::look_at_rh(light_position, cascade_center, light_up));

            let mut min_z = f32::MAX;
            let mut max_z = f32::MIN;

            for corner in &cascade_corners {
                let light_point = light_view.value * corner.extend(1.0);

                min_z = min_z.min(light_point.z);
                max_z = max_z.max(light_point.z);
            }

            let shadow_near = -max_z;
            let shadow_far = -min_z;

            let projection = Self::build_bound_to_texel_orthographic(
                shadow_map_resolution,
                cascade_radius,
                shadow_near,
                shadow_far,
            );

            ViewProjectionMatrix::from_view_projection(&light_view, &projection).vulkan_corrected()
        }).collect::<Vec<_>>()
    }

    fn get_cascade_corners(
        original_corners: &[Vec3],
        range: &Range<f32>,
        camera_near: f32,
        camera_far: f32,
    ) -> Vec<Vec3> {
        let mut cascade_corners = Vec::with_capacity(8);

        let z_length = camera_far - camera_near;
        let t_start = (range.start - camera_near) / z_length;
        let t_end = (range.end - camera_near) / z_length;

        for i in 0..4 {
            let near = original_corners[i * 2];
            let far = original_corners[i * 2 + 1];
            let ray_direction = far - near;

            cascade_corners.push(near + ray_direction * t_start);
            cascade_corners.push(near + ray_direction * t_end);
        }

        cascade_corners
    }

    fn get_frustum_center(corners: &Vec<Vec3>) -> Vec3 {
        let mut frustum_center = Vec3::ZERO;

        for corner in corners {
            frustum_center += *corner;
        }

        frustum_center / corners.len() as f32
    }

    fn get_frustum_radius(corners: &Vec<Vec3>, center: Vec3) -> f32 {
        let mut radius = 0.0_f32;

        for corner in corners {
            radius = radius.max(corner.distance(center));
        }

        radius.ceil()
    }

    fn get_frustum_corners(view_projection: &ViewProjectionMatrix) -> Vec<Vec3> {
        let inverted_projection = view_projection.value.inverse();

        let mut corners = Vec::with_capacity(8);

        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                for z in [0.0, 1.0] {
                    let point = inverted_projection * Vec4::new(x, y, z, 1.0);

                    corners.push(point.xyz() / point.w);
                }
            }
        }

        corners
    }

    fn build_bound_to_texel_orthographic(
        shadow_map_resolution: u32,
        cascade_radius: f32,
        shadow_near: f32,
        shadow_far: f32,
    ) -> ProjectionMatrix {
        let cascade_diameter = cascade_radius * 2.0;
        let world_units_per_texel = cascade_diameter / shadow_map_resolution as f32;

        let min_x = (-cascade_radius / world_units_per_texel).floor() * world_units_per_texel;
        let min_y = (-cascade_radius / world_units_per_texel).floor() * world_units_per_texel;

        let max_x = min_x + cascade_diameter;
        let max_y = min_y + cascade_diameter;

        ProjectionMatrix::from_mat4(
            Mat4::orthographic_rh(
                min_x, max_x,
                min_y, max_y,
                shadow_near, shadow_far,
            )
        )
    }
}
