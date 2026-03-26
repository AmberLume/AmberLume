use glam::Vec3;
use rapier3d::math::Point;
use rapier3d::pipeline::{DebugColor, DebugRenderBackend};
use rapier3d::prelude::DebugRenderObject;

pub struct PhysicsDebugRender {
    pub position: Vec3,
    pub radius: f32,

    pub lines: Vec<PhysicsDebugLine>,
}

impl PhysicsDebugRender {
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

impl DebugRenderBackend for PhysicsDebugRender {
    fn filter_object(&self, object: DebugRenderObject) -> bool {
        let position = match object {
            DebugRenderObject::RigidBody(_, _) => return false,
            DebugRenderObject::Collider(_, collider) => {
                let translation = collider.position().translation;

                Vec3::new(translation.x, translation.y, translation.z)
            },
            DebugRenderObject::ColliderAabb(_, _, _) => return false,
            DebugRenderObject::ImpulseJoint(_, _) => return false,
            DebugRenderObject::MultibodyJoint(_, _, _) => return false,
            DebugRenderObject::ContactPair(_, _, _) => return false,
        };

        position.distance(self.position) <= self.radius
    }

    fn draw_line(&mut self, _object: DebugRenderObject, start: Point<f32>, end: Point<f32>, color: DebugColor) {
        let debug_line = PhysicsDebugLine {
            start: [start.x, start.y, start.z],
            end: [end.x, end.y, end.z],

            color,
        };

        self.lines.push(debug_line);
    }
}

#[derive(Debug, Clone)]
pub struct PhysicsDebugLine {
    pub start: [f32; 3],
    pub end: [f32; 3],

    pub color: [f32; 4],
}
