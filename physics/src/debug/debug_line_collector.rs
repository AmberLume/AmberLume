use math::{Vec3, Vec4};
use rapier3d::math::Vector;
use rapier3d::pipeline::{DebugColor, DebugRenderBackend};
use rapier3d::prelude::DebugRenderObject;
use crate::debug::PhysicsDebugLine;
use crate::rapier_math::to_math::ToMath;

pub(crate) struct DebugLineCollector {
    pub position: Vec3,
    pub radius: f32,

    pub lines: Vec<PhysicsDebugLine>,
}

impl DebugLineCollector {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            radius: 0.0,

            lines: Vec::new(),
        }
    }

    pub fn set_position_radius(&mut self, position: &Vec3, radius: f32) {
        self.position = *position;
        self.radius = radius;
    }

    pub fn reset(&mut self) {
        self.lines.clear();
    }
}

impl DebugRenderBackend for DebugLineCollector {
    fn filter_object(&self, object: DebugRenderObject) -> bool {
        let position = match object {
            DebugRenderObject::RigidBody(_, _) => return false,
            DebugRenderObject::Collider(_, collider) => collider.position().translation.to_math(),
            DebugRenderObject::ColliderAabb(_, _, _) => return false,
            DebugRenderObject::ImpulseJoint(_, _) => return false,
            DebugRenderObject::MultibodyJoint(_, _, _) => return false,
            DebugRenderObject::ContactPair(_, _, _) => return false,
        };

        position.distance(self.position) <= self.radius
    }

    fn draw_line(&mut self, _object: DebugRenderObject, start: Vector, end: Vector, color: DebugColor) {
        let debug_line = PhysicsDebugLine {
            start: start.to_math(),
            end: end.to_math(),

            color: Vec4::from_array(color),
        };

        self.lines.push(debug_line);
    }
}
