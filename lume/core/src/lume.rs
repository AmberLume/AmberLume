use crate::engine::systems::camera_system::camera_system;
use crate::scene::scene_manager::SceneManager;
use amber_lume::amber_lume::AmberLume;
use amber_lume::animation::animation_states::humanoid_animation_state::HumanoidAnimationState;
use amber_lume::platform_providers::providers::Providers;
use amber_lume::limits::AmberLumeLimits;
use amber_lume::settings::settings::EngineSettings;
use amber_lume::world::physics::systems::character_physics_force_system::character_physics_force_system;
use amber_lume::world::physics::systems::physics_iterator_system::physics_iterator_system;
use amber_lume::world::physics::systems::physics_registration_system::physics_registration_system;
use amber_lume::world::physics::systems::physics_synchronization_system::physics_synchronization_system;
use amber_lume::world::systems::animation_mapping_system::humanoid_animation_system;
use amber_lume::world::systems::animation_resolver_system::animation_resolver_system;
use amber_lume::world::systems::animation_system::animation_system;
use amber_lume::world::systems::render_snapshot_system::render_snapshot_system;
use amber_lume::world::systems::resource_resolver_system::resource_resolver_system;
use amber_lume::world::systems::time_system::world_time_system;
use amber_lume::world::systems::user_input_system::user_input_system;
use amber_lume::world::systems::world_day_night_system::world_day_night_system;
use amber_lume::world::unique::user_input_unique::UserInputUnique;
use anyhow::Result;
use shipyard::{EntitiesView, UniqueViewMut, Workload};
use std::sync::Arc;
use amber_lume::input_handler::hardware_pointer_event::HardwarePointerEvent;
use amber_lume::input_handler::hardware_key_codes::HardwareKeyCode;
use amber_lume::input_handler::input_frame::PointerId;
use amber_lume::render::device::layers::VulkanLayer;
use amber_lume::ui::ui_renderer::UiRenderer;

pub struct Lume {
    amber_lume: AmberLume,
}

impl Lume {
    pub fn create(
        providers: Providers,
        limits: AmberLumeLimits,
        layers: Vec<VulkanLayer>,
        ui_renderer: Arc<dyn UiRenderer>,
    ) -> Result<Self> {
        let amber_lume = AmberLume::new(
            providers,
            ui_renderer.clone(),
            limits,
            layers,
            EngineSettings::default(),
        )?;

        let scene_loader = amber_lume.get_scene_loader();
        let scene_manager = SceneManager::create(scene_loader);
        scene_manager.set_test_scene(&amber_lume.world);

        Self::bind_workloads(&amber_lume)?;

        Ok(Self { amber_lume })
    }

    fn bind_workloads(amber_lume: &AmberLume) -> Result<()> {
        Workload::new("common")
            .with_system(world_time_system)
            .with_system(user_input_system)
            .with_system(physics_registration_system)
            .with_system(physics_iterator_system)
            .with_system(character_physics_force_system)
            .with_system(physics_synchronization_system)
            .with_system(camera_system)
            .with_system(resource_resolver_system)
            .with_system(animation_resolver_system)
            .with_system(humanoid_animation_system)
            .with_system(animation_system::<HumanoidAnimationState>)
            .with_system(world_day_night_system)
            .with_system(render_snapshot_system)
            .add_to_world(&amber_lume.world)?;

        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        self.update_world()?;

        self.amber_lume.render()
    }

    fn update_world(&mut self) -> Result<()> {
        let input_frame = self.amber_lume.handle_input();

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
        self.amber_lume.on_hardware_pointer_button(&id, event);
    }

    pub fn push_hardware_keycode_event(&mut self, keycode: HardwareKeyCode, pressed: bool) {
        self.amber_lume.on_hardware_input(keycode, pressed);
    }

    pub fn on_update_surface(&mut self) {
        self.amber_lume.set_swapchain_out_of_date()
    }

    pub fn on_close(self) -> Result<()> {
        self.amber_lume.stop()
    }
}
