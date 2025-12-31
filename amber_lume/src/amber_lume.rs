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
use anyhow::Result;
use shipyard::World;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::info;

pub struct AmberLume {
    vulkan_context: Arc<VulkanContext>,
    vulkan_surface: VulkanSurface,

    device_context: DeviceContext,
    swapchain_context: SwapchainContext,

    world_snapshot_handler: Arc<WorldSnapshotHandler>,

    pub world: World,

    renderer: Renderer,

    resource_context: ResourceContext,

    providers: Providers,

    resource_hub: Arc<ResourceHub>,
}

impl AmberLume {
    pub fn new(providers: Providers) -> Result<Self> {
        let context_profile = ContextProfile::from(providers.surface_provider.clone())?;

        let vulkan_context = Arc::new(VulkanContext::new(context_profile)?);

        let vulkan_surface =
            VulkanSurface::create(&vulkan_context, providers.surface_provider.clone())?;

        let mut device_context = DeviceContext::new(&vulkan_context, &vulkan_surface)?;

        let swapchain_context = SwapchainContext::create(
            &vulkan_context,
            &vulkan_surface,
            &device_context,
            providers.surface_provider.clone(),
        )?;

        let mut resource_context = ResourceContext::create(&mut device_context)?;
        let resource_hub = {
            let resource_hub = ResourceHub::create(
                &mut device_context,
                &mut resource_context,
                providers.io_provider.clone(),
            )?;

            Arc::new(resource_hub)
        };

        let renderer = Renderer::create(
            &vulkan_context,
            &mut device_context,
            &resource_context,
            &swapchain_context,
            resource_hub.clone(),
        )?;

        let world_snapshot_handler = Arc::new(WorldSnapshotHandler::new());

        let world = World::new();
        world.add_unique(WorldTimeUnique::new());
        world.add_unique(WorldCameraUnique::new());
        world.add_unique(WorldSnapshotUnique::new(world_snapshot_handler.clone()));
        world.add_unique(ResourceResolverUnique::new(resource_hub.clone()));

        info!("AmberLume created");

        Ok(Self {
            vulkan_context,
            vulkan_surface,

            device_context,
            swapchain_context,

            world_snapshot_handler,

            world,

            renderer,

            resource_context,

            providers,

            resource_hub,
        })
    }

    pub fn render(&mut self) -> Result<()> {
        let (width, height) = self.providers.surface_provider.size();
        if width == 0 || height == 0 {
            return Ok(());
        }

        if self
            .swapchain_context
            .is_out_of_date
            .load(Ordering::Relaxed)
        {
            self.invalidate_swapchain()?;
        }

        let world_snapshot = self.world_snapshot_handler.pull();
        self.renderer.render_frame(
            &self.device_context,
            &self.swapchain_context,
            world_snapshot,
        )?;

        Ok(())
    }

    pub fn invalidate_swapchain(&mut self) -> Result<()> {
        info!("Invalidating swapchain");

        self.device_context.queues.present_wait_idle()?;

        self.renderer.teardown(&mut self.device_context)?;

        self.swapchain_context.teardown_and_setup(
            &self.vulkan_context,
            &self.vulkan_surface,
            &mut self.device_context,
            self.providers.surface_provider.clone(),
        )?;

        self.renderer.setup(
            &self.vulkan_context,
            &mut self.device_context,
            &self.swapchain_context,
        )?;

        self.swapchain_context.set_is_out_of_date(false);

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

        self.renderer.destroy(&mut self.device_context)?;

        let hub = Arc::try_unwrap(self.resource_hub)
            .map_err(|arc| anyhow::anyhow!("ResourceHub refs: {}", Arc::strong_count(&arc)))?;
        hub.destroy()?;

        self.swapchain_context.destroy(&mut self.device_context)?;
        self.resource_context.destroy(&self.device_context)?;

        self.device_context.destroy()?;

        self.vulkan_surface.destroy(&self.vulkan_context)?;
        self.vulkan_context.destroy()?;

        info!("AmberLume destroyed");

        Ok(())
    }
}
