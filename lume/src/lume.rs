use crate::engine::systems::camera_system::camera_system;
use crate::platform_providers::desktop_io_provider::DesktopIOProvider;
use crate::platform_providers::surface_provider::VulkanSurfaceProvider;
use crate::scene::scene_manager::SceneManager;
use amber_lume::amber_lume::AmberLume;
use amber_lume::platform_providers::providers::Providers;
use amber_lume::world::systems::resource_resolver_system::resource_resolver_system;
use amber_lume::world::systems::time_system::world_time_system;
use amber_lume::world::systems::world_snapshot_system::world_snapshot_system;
use anyhow::Result;
use shipyard::{UniqueViewMut, Workload};
use std::sync::Arc;
use winit::window::Window;
use amber_lume::input_handler::input_event::KeyEvent;
use amber_lume::world::physics::systems::character_physics_force_system::character_physics_force_system;
use amber_lume::world::physics::systems::physics_iterator_system::physics_iterator_system;
use amber_lume::world::physics::systems::physics_registration_system::physics_registration_system;
use amber_lume::world::physics::systems::physics_synchronization_system::physics_synchronization_system;
use amber_lume::world::systems::user_input_system::user_input_system;
use amber_lume::world::unique::user_input_unique::UserInputUnique;
use crate::ui::ui_renderer::LumeUiRenderer;

pub struct Lume {
    amber_lume: AmberLume,
}

impl Lume {
    pub fn create(window: Arc<Window>) -> Result<Self> {
        let providers = Providers {
            io_provider: Arc::new(DesktopIOProvider::new()),
            surface_provider: Arc::new(VulkanSurfaceProvider::new(window.clone())),
        };

        let ui_renderer = Box::new(LumeUiRenderer::new());

        let amber_lume = AmberLume::new(providers, ui_renderer)?;
        
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
            .with_system(world_snapshot_system)
            .add_to_world(&amber_lume.world)?;
        
        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        self.update_world()?;

        self.amber_lume.render()
    }

    fn update_world(&mut self) -> Result<()> {
        let world = &self.amber_lume.world;

        let (state, events) = self.amber_lume.input_handler.pull();

        world.run(|mut user_input: UniqueViewMut<UserInputUnique>| {
            user_input.state = state;

            user_input.events.clear();
            user_input.events.extend_from_slice(events);
        });

        world.run_workload("common")?;

        Ok(())
    }

    pub fn on_key_event(&mut self, key_event: KeyEvent) {
        self.amber_lume.input_handler.push(key_event);
    }

    pub fn on_update_surface(&mut self) -> Result<()> {
        self.amber_lume.invalidate_swapchain()
    }

    pub fn on_close(self) -> Result<()> {
        self.amber_lume.stop()
    }
}
