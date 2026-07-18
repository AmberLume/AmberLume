use std::sync::Arc;
use arc_swap::ArcSwap;
use glam::Vec3;
use physics::{PhysicsConfig, PhysicsContext, PhysicsDebugRenderer};
use shipyard::Unique;
use crate::settings::settings::EngineSettings;

#[derive(Unique)]
pub struct PhysicsContextUnique {
    pub handle: PhysicsContext,
    pub debug_renderer: PhysicsDebugRenderer,

    pub settings: Arc<ArcSwap<EngineSettings>>,
    pub fixed_delta_time: f32,

    pub iterate_count: u32,
}

impl PhysicsContextUnique {
    pub const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);

    pub fn new(
        settings: Arc<ArcSwap<EngineSettings>>,
        fixed_delta_time: f32,
    ) -> Self {
        let config = Self::build_config(&settings, fixed_delta_time);
        let handle = PhysicsContext::new(config);

        Self {
            handle,
            debug_renderer: PhysicsDebugRenderer::new(),

            settings,
            fixed_delta_time,

            iterate_count: 0,
        }
    }

    pub fn refresh_config(&mut self) {
        let config = Self::build_config(&self.settings, self.fixed_delta_time);

        self.handle.configure(config);
    }

    fn build_config(
        settings: &Arc<ArcSwap<EngineSettings>>,
        fixed_delta_time: f32,
    ) -> PhysicsConfig {
        let settings = settings.load();

        PhysicsConfig {
            gravity: Self::GRAVITY,
            fixed_delta_time,

            paused: settings.debug.physics_paused.value,
            interpolation: settings.debug.physics_interpolation.value,
            render_colliders: settings.debug.collider_rendering_enabled.value,
        }
    }
}
