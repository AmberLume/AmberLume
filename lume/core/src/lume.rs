use crate::scene::scene_manager::SceneManager;
use amber_lume::amber_lume::AmberLume;
use amber_lume::animation::animation_states::humanoid_animation_state::HumanoidAnimationState;
use amber_lume::lifecycle::lifecycle::AmberLumeLifecycle;
use amber_lume::platform_providers::surface_provider::SurfaceProvider;
use amber_lume::world::physics::systems::physics_deregistration_system::physics_deregistration_system;
use amber_lume::world::physics::systems::physics_registration_system::physics_registration_system;
use amber_lume::world::physics::systems::physics_step_system::physics_step_system;
use amber_lume::world::physics::systems::physics_synchronization_system::physics_synchronization_system;
use amber_lume::world::systems::animation_mapping_system::humanoid_animation_system;
use amber_lume::world::systems::animation_resolver_system::animation_resolver_system;
use amber_lume::world::systems::animation_system::animation_system;
use amber_lume::world::systems::camera_fly_system::camera_fly_system;
use amber_lume::world::systems::camera_sync_system::camera_synchronization_system;
use amber_lume::world::systems::mouse_look_system::mouse_look_system;
use amber_lume::world::systems::render_snapshot_system::render_snapshot_system;
use amber_lume::world::systems::render_view_resolve_system::render_view_resolve_system;
use amber_lume::world::systems::resource_resolver_system::resource_resolver_system;
use amber_lume::world::systems::time_system::world_time_system;
use amber_lume::world::systems::user_input_system::user_input_system;
use amber_lume::world::systems::global_light_system::global_light_system;
use amber_lume::world::unique::user_input_unique::UserInputUnique;
use anyhow::Result;
use shipyard::{EntitiesView, UniqueViewMut, Workload};
use std::sync::Arc;
use amber_lume::input_handler::hardware_pointer_event::HardwarePointerEvent;
use amber_lume::input_handler::hardware_key_codes::HardwareKeyCode;
use amber_lume::input_handler::input_frame::{InputFrame, PointerId};
use amber_lume::render::target::render_target::RenderTarget;
use amber_lume::settings::settings_handler::EngineSettingsHandler;

pub struct Lume {
    amber_lume: AmberLume,
}

impl Lume {
    pub fn new(amber_lume: AmberLume) -> Result<Self> {
        let scene_manager = SceneManager::create(amber_lume.scene_loader.clone());
        scene_manager.set_test_scene(&amber_lume.world);

        Self::workloads().add_to_world(&amber_lume.world)?;

        Ok(Self {
            amber_lume,
        })
    }

    fn workloads() -> Workload {
        Workload::new("common")
            .with_system(world_time_system)
            .with_system(user_input_system)
            .with_system(physics_registration_system)
            .with_system(physics_step_system)
            .with_system(physics_synchronization_system)
            .with_system(physics_deregistration_system)
            .with_system(mouse_look_system)
            .with_system(camera_fly_system)
            .with_system(camera_synchronization_system)
            .with_system(render_view_resolve_system)
            .with_system(resource_resolver_system)
            .with_system(animation_resolver_system)
            .with_system(humanoid_animation_system)
            .with_system(animation_system::<HumanoidAnimationState>)
            .with_system(global_light_system)
            .with_system(render_snapshot_system)
    }

    pub fn draw(&mut self) -> Result<()> {
        let input_frame = self.amber_lume.handle_input();

        self.amber_lume.render_ui(&input_frame);
        self.update_world(input_frame)?;

        self.amber_lume.render()
    }

    fn update_world(&mut self, input_frame: InputFrame) -> Result<()> {
        let world = &self.amber_lume.world;

        world.run(|mut user_input: UniqueViewMut<UserInputUnique>| {
            user_input.input_frame = input_frame;
        });

        let mut entity_count: u32 = 0;
        world.run(|entities: EntitiesView| {
            entity_count = entities.iter().count() as u32;
        });

        world.run_workload("common")?;

        Ok(())
    }

    pub fn push_hardware_pointer_event(&mut self, id: &PointerId, event: HardwarePointerEvent) {
        self.amber_lume.on_hardware_pointer_button(id, event);
    }

    pub fn push_hardware_keycode_event(&mut self, keycode: HardwareKeyCode, pressed: bool) {
        self.amber_lume.on_hardware_input(keycode, pressed);
    }

    pub fn on_update_surface(&mut self) {
        self.amber_lume.set_render_target_out_of_date();
    }

    pub fn engine_settings(&self) -> &EngineSettingsHandler {
        self.amber_lume.settings_handler()
    }

    pub fn create_surface_target(
        &self,
        surface_provider: Arc<dyn SurfaceProvider>,
    ) -> Result<Arc<dyn RenderTarget>> {
        self.amber_lume.create_surface_target(surface_provider)
    }

    pub fn destroy(self) -> Result<()> {
        self.amber_lume.destroy()?;
        
        Ok(())
    }
}

impl AmberLumeLifecycle for Lume {
    fn attach_render_target(&mut self, target: Arc<dyn RenderTarget>) -> Result<()> {
        self.amber_lume.attach_render_target(target)?;

        Ok(())
    }

    fn detach_render_target(&mut self) -> Result<()> {
        self.amber_lume.detach_render_target()
    }

    fn is_render_target_attached(&self) -> bool {
        self.amber_lume.is_render_target_attached()
    }

    fn pause(&mut self) {
        self.amber_lume.pause();
    }

    fn resume(&mut self) {
        self.amber_lume.resume();
    }

    fn is_paused(&self) -> bool {
        self.amber_lume.is_paused()
    }
}
