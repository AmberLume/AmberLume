use rapier3d::math::Point;
use rapier3d::pipeline::{DebugColor, DebugRenderBackend};
use rapier3d::prelude::DebugRenderObject;

pub struct PhysicsDebugRender {
    pub lines: Vec<PhysicsDebugLine>,
}

impl PhysicsDebugRender {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn reset(&mut self) {
        self.lines.clear();
    }
}

impl DebugRenderBackend for PhysicsDebugRender {
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
