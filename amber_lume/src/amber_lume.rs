use std::mem::replace;
use crate::platform_providers::providers::Providers;
use crate::render::context_profile::ContextProfile;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::renderer::renderer::Renderer;
use crate::render::vulkan::surface::vulkan_surface::VulkanSurface;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use crate::render::vulkan::vulkan_context::VulkanContext;
use crate::resources::resource_hub::ResourceHub;
use crate::snapshot_handler::world_snapshot_handler::WorldSnapshotHandler;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use crate::world::unique::world_camera_unique::WorldCameraUnique;
use crate::world::unique::world_snapshot_unique::WorldSnapshotUnique;
use crate::world::unique::world_time_unique::WorldTimeUnique;
use anyhow::{anyhow, Result};
use shipyard::World;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use ash::vk::SampleCountFlags;
use tracing::info;
use crate::input_handler::input_handler::InputHandler;
use crate::limits::renderer_limits::RendererLimits;
use crate::resources::descriptor_index_managers::IndexManagers;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::resources::resource_factories::ResourceFactories;
use crate::ui::ui_context::UiContext;
use crate::resources::scene_loader::scene_loader::SceneLoader;
use crate::system_stats::SystemStatsHolder;
use crate::ui::ui_renderer::UiRenderer;
use crate::world::physics::physics_world_unique::PhysicsWorldUnique;
use crate::world::unique::global_shadow_unique::GlobalShadowUnique;
use crate::world::unique::user_input_unique::UserInputUnique;

pub struct AmberLume {
    vulkan_context: Arc<VulkanContext>,
    vulkan_surface: VulkanSurface,

    renderer_limits: RendererLimits,

    device_context: DeviceContext,
    swapchain_context: SwapchainContext,

    pub input_handler: InputHandler,

    ui_context: UiContext,

    world_snapshot_handler: Arc<WorldSnapshotHandler>,

    pub world: World,

    renderer: Renderer,

    resource_context: ResourceContext,

    providers: Providers,

    index_managers: Arc<IndexManagers>,
    resource_factories: Arc<ResourceFactories>,
    persistent_resources: Arc<PersistentResources>,
    resource_hub: Arc<ResourceHub>,

    pub system_stats_handler: SystemStatsHolder,

    frame_counter: Arc<AtomicU64>,
}

impl AmberLume {
    pub fn new(
        providers: Providers,
        ui_renderer: Box<dyn UiRenderer>,
        renderer_limits: RendererLimits,
    ) -> Result<Self> {
        let frame_counter = Arc::new(AtomicU64::new(0));

        let context_profile = ContextProfile::from(providers.surface_provider.clone())?;

        let vulkan_context = Arc::new(VulkanContext::new(context_profile)?);

        let vulkan_surface = VulkanSurface::create(&vulkan_context, providers.surface_provider.clone())?;

        let mut device_context = DeviceContext::new(&vulkan_context, &vulkan_surface)?;

        let swapchain_context = SwapchainContext::create(
            None,
            &vulkan_context,
            &vulkan_surface,
            &device_context,
            providers.surface_provider.clone(),
        )?;

        let descriptor_index_managers = Arc::new(
            IndexManagers::create(
                &renderer_limits,
                swapchain_context.swapchain_images.len() as u64,
                frame_counter.clone(),
            ));

        let resource_factories = Arc::new(
            ResourceFactories::create(
                &device_context,
            )?
        );
        let mut resource_context = ResourceContext::create(
            &device_context.device,
            device_context.queues.clone(),
            resource_factories.clone(),
            &renderer_limits,
        )?;
        let persistent_resources = Arc::new(PersistentResources::create(
            resource_context.resource_loader.clone(),
            &resource_factories,
            &renderer_limits,
            &resource_context.buffer_manager,
            &descriptor_index_managers,
            swapchain_context.format,
            SampleCountFlags::TYPE_1,
        )?);
        let resource_hub = {
            let resource_hub = ResourceHub::create(
                &mut device_context,
                &mut resource_context,
                descriptor_index_managers.clone(),
                frame_counter.clone(),
                resource_factories.clone(),
                persistent_resources.clone(),
                providers.io_provider.clone(),
            )?;

            Arc::new(resource_hub)
        };

        let renderer = Renderer::create(
            &vulkan_context.instance,
            &device_context.device,
            &renderer_limits,
            device_context.physical_device_info.handle,
            &device_context.queues,
            &descriptor_index_managers,
            &resource_factories,
            &resource_context,
            &swapchain_context,
            resource_hub.clone(),
            persistent_resources.clone(),
        )?;

        let input_handler = InputHandler::create();

        let ui_context = UiContext::new(
            resource_factories.clone(),
            descriptor_index_managers.clone(),
            persistent_resources.clone(),
            resource_context.buffer_manager.clone(),
            resource_context.resource_loader.clone(),
            ui_renderer,
        )?;

        let world_snapshot_handler = Arc::new(WorldSnapshotHandler::new());

        let world = World::new();
        world.add_unique(UserInputUnique::new());
        world.add_unique(WorldTimeUnique::new());
        world.add_unique(WorldCameraUnique::new());
        world.add_unique(GlobalShadowUnique::new());
        world.add_unique(WorldSnapshotUnique::new(world_snapshot_handler.clone()));
        world.add_unique(ResourceResolverUnique::new(resource_hub.clone()));
        world.add_unique(PhysicsWorldUnique::new());

        let system_stats_handler = SystemStatsHolder::create();

        info!("AmberLume created");

        Ok(Self {
            vulkan_context,
            vulkan_surface,

            renderer_limits,

            device_context,
            swapchain_context,

            input_handler,

            ui_context,

            world_snapshot_handler,

            world,

            renderer,

            resource_context,

            providers,

            index_managers: descriptor_index_managers,
            resource_factories,
            persistent_resources,
            resource_hub,

            system_stats_handler,

            frame_counter,
        })
    }
    
    pub fn get_scene_loader(&self) -> Arc<SceneLoader> {
        self.resource_hub.scene_loader.clone()
    }

    pub fn render(&mut self) -> Result<()> {
        let (width, height) = self.providers.surface_provider.size();
        if width == 0 || height == 0 {
            return Ok(());
        }

        if self.swapchain_context.is_out_of_date.load(Ordering::Relaxed) {
            self.invalidate_swapchain()?;
        }

        self.ui_context.render_ui(
            self.swapchain_context.extent,
            &self.system_stats_handler.get_snapshot(),
        );

        let world_snapshot = self.world_snapshot_handler.pull();
        self.renderer.render_frame(
            &self.device_context,
            &self.swapchain_context,
            &mut self.ui_context,
            &self.renderer_limits,
            &self.resource_context.buffer_manager,
            &mut self.system_stats_handler,
            world_snapshot,
        )?;

        self.system_stats_handler.publish();
        
        self.resource_hub.update();

        self.frame_counter.fetch_add(1, Ordering::Relaxed);
        self.index_managers.update();

        Ok(())
    }

    pub fn invalidate_swapchain(&mut self) -> Result<()> {
        info!("Invalidating swapchain");

        self.device_context.queues.present_wait_idle()?;

        let new_swapchain_context = SwapchainContext::create(
            Some(&self.swapchain_context),
            &self.vulkan_context,
            &self.vulkan_surface,
            &mut self.device_context,
            self.providers.surface_provider.clone(),
        )?;
        let new_renderer = Renderer::create(
            &self.vulkan_context.instance,
            &self.device_context.device,
            &self.renderer_limits,
            self.device_context.physical_device_info.handle,
            &self.device_context.queues,
            &self.index_managers,
            &self.resource_factories,
            &self.resource_context,
            &new_swapchain_context,
            self.resource_hub.clone(),
            self.persistent_resources.clone(),
        )?;

        let old_swapchain_context = replace(&mut self.swapchain_context, new_swapchain_context);
        let old_renderer = replace(&mut self.renderer, new_renderer);

        old_swapchain_context.destroy(&self.device_context.device)?;
        old_renderer.destroy(&self.device_context.device, &self.index_managers, &self.resource_factories)?;

        info!("Swapchain invalidated");

        Ok(())
    }

    pub fn stop(self) -> Result<()> {
        info!("Stop running");

        self.destroy()?;

        info!("AmberLume stopped gracefully");

        Ok(())
    }

    pub fn destroy(mut self) -> Result<()> {
        self.device_context.queues.all_wait_idle()?;

        self.world.clear();
        self.world.remove_unique::<ResourceResolverUnique>()?;

        self.ui_context.destroy()?;

        self.renderer.destroy(&self.device_context.device, &self.index_managers, &self.resource_factories)?;

        let hub = Arc::try_unwrap(self.resource_hub).map_err(|arc|
            anyhow!("ResourceHub refs: {}", Arc::strong_count(&arc))
        )?;
        hub.destroy()?;

        let persistent_resources = Arc::try_unwrap(self.persistent_resources).map_err(|arc|
            anyhow!("PersistentResources refs: {}", Arc::strong_count(&arc))
        )?;
        persistent_resources.destroy(
            &self.index_managers,
            &self.resource_factories,
        )?;

        self.swapchain_context.destroy(&self.device_context.device)?;
        self.resource_context.destroy(&self.resource_factories.managed_buffer_factory)?;

        let resource_factories = Arc::try_unwrap(self.resource_factories).map_err(|arc|
            anyhow!("ResourceFactories refs: {}", Arc::strong_count(&arc))
        )?;
        resource_factories.destroy()?;

        self.device_context.destroy()?;

        self.vulkan_surface.destroy(&self.vulkan_context)?;
        self.vulkan_context.destroy()?;

        info!("AmberLume destroyed");

        Ok(())
    }
}
