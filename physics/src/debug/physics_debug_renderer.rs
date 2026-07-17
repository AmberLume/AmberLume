use glam::Vec3;
use rapier3d::prelude::{DebugRenderMode, DebugRenderPipeline, DebugRenderStyle};
use crate::context::PhysicsContext;
use crate::debug::{DebugLineCollector, PhysicsDebugLine};

pub struct PhysicsDebugRenderer {
    collector: DebugLineCollector,
    pipeline: DebugRenderPipeline,
}

impl PhysicsDebugRenderer {
    pub fn new() -> Self {
        Self {
            collector: DebugLineCollector::new(),
            pipeline: DebugRenderPipeline::new(
                DebugRenderStyle::default(),
                DebugRenderMode::COLLIDER_SHAPES,
            ),
        }
    }

    pub fn fill(&mut self, context: &PhysicsContext, position: Vec3, radius: f32) {
        if !context.config.render_colliders {
            return;
        }

        self.collector.reset();
        self.collector.set_position_radius(&position, radius);

        self.pipeline.render(
            &mut self.collector,
            &context.rigid_body_set,
            &context.collider_set,
            &context.impulse_joint_set,
            &context.multibody_joint_set,
            &context.narrow_phase,
        );
    }

    pub fn lines(&self) -> &[PhysicsDebugLine] {
        &self.collector.lines
    }
}
