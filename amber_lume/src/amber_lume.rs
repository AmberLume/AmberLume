use std::mem::replace;
use crate::platform_providers::providers::Providers;
use crate::render::builder::context_profile::ContextProfile;
use crate::render::resources::resource_context::ResourceContext;
use crate::render::device::device_context::DeviceContext;
use crate::render::render::Render;
use crate::render::surface::render_surface::RenderSurface;
use crate::render::swapchain::swapchain_context::SwapchainContext;
use crate::render::device::vulkan_context::VulkanContext;
use crate::resources::resource_hub::ResourceHub;
use crate::snapshot_handler::render_snapshot_handler::RenderSnapshotHandler;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use crate::world::unique::render_snapshot_unique::RenderSnapshotUnique;
use crate::world::unique::world_time_unique::WorldTimeUnique;
use anyhow::Result;
use shipyard::World;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{info, warn};
use crate::input_handler::input_handler::InputHandler;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::device::layers::VulkanLayer;
use crate::resources::index_managers::IndexManagers;
use crate::resources::descriptor_set_manager::DescriptorSetManager;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::render_graph::image_state_tracker::image_state_tracker::ImageStateTracker;
use crate::resources::alpaca_resource_reader::AlpacaResourceReader;
use crate::ui::ui_context::UiContext;
use crate::resources::scene_loader::SceneLoader;
use crate::settings::settings::EngineSettings;
use crate::settings::settings_handler::EngineSettingsHandler;
use crate::statistics::amber_lume_statistics::AmberLumeStatistics;
use crate::ui::events::ui_events::MouseEvent;
use crate::ui::ui_renderer::UiRenderer;
use crate::utils::arc_utils::ArcUnwrapOrErr;
use crate::world::physics::physics_world_unique::PhysicsWorldUnique;
use crate::world::unique::global_shadow_unique::GlobalShadowUnique;
use crate::world::unique::render_view_unique::RenderViewUnique;
use crate::world::unique::resource_loader_unique::ResourceLoaderUnique;
use crate::world::unique::user_input_unique::UserInputUnique;

pub struct AmberLume {
    vulkan_context: Arc<VulkanContext>,
    render_surface: RenderSurface,

    renderer_limits: RendererLimits,
    settings_handler: EngineSettingsHandler,

    device_context: DeviceContext,
    swapchain_context: SwapchainContext,

    pub input_handler: InputHandler,

    ui_context: UiContext,

    render_snapshot_handler: Arc<RenderSnapshotHandler>,

    pub world: World,

    renderer: Render,

    resource_context: ResourceContext,

    providers: Providers,

    scene_loader: Arc<SceneLoader>,
    
    index_managers: Arc<IndexManagers>,
    resource_factories: Arc<ResourceFactories>,
    resource_hub: Arc<ResourceHub>,

    image_state_tracker: ImageStateTracker,

    frame_counter: Arc<AtomicU64>,
}

impl AmberLume {
    pub fn new(
        providers: Providers,
        ui_renderer: Arc<dyn UiRenderer>,
        renderer_limits: RendererLimits,
        layers: Vec<VulkanLayer>,
        engine_settings: EngineSettings,
    ) -> Result<Self> {
        let resource_reader = Arc::new(AlpacaResourceReader::new(providers.io_provider.clone())?);
        
        let settings_handler = EngineSettingsHandler::new(engine_settings);

        let frame_counter = Arc::new(AtomicU64::new(0));

        let context_profile = ContextProfile::from(
            providers.surface_provider.clone(),
            layers,
        )?;

        let vulkan_context = Arc::new(VulkanContext::new(context_profile)?);

        let render_surface = RenderSurface::create(&vulkan_context, providers.surface_provider.clone())?;

        let mut device_context = DeviceContext::new(&vulkan_context, &render_surface)?;

        let swapchain_context = SwapchainContext::create(
            None,
            &vulkan_context,
            &render_surface,
            &device_context,
            providers.surface_provider.clone(),
        )?;
        
        let descriptor_index_managers = Arc::new(IndexManagers::create(
            &renderer_limits,
            swapchain_context.swapchain_images.len() as u32,
            frame_counter.clone(),
        ));

        let resource_factories = Arc::new(ResourceFactories::create(&device_context)?);
        let mut resource_context = ResourceContext::create(
            &device_context.device,
            device_context.queues.clone(),
            resource_factories.clone(),
            &renderer_limits,
        )?;

        let descriptor_set_manager = Arc::new(DescriptorSetManager::new(
            device_context.device.clone(),
            &resource_factories.descriptor_set_layout_factory,
            &resource_factories.descriptor_set_factory,
            &resource_factories.sampler_factory,
            &renderer_limits,
        )?);

        let mut image_state_tracker = ImageStateTracker::new();

        let resource_hub = Arc::new(ResourceHub::create(
            &mut device_context,
            &mut resource_context,
            &swapchain_context,
            &renderer_limits,
            resource_reader.clone(),
            descriptor_set_manager.clone(),
            descriptor_index_managers.clone(),
            frame_counter.clone(),
            resource_factories.clone(),
            &mut image_state_tracker,
        )?);
        
        let renderer = Render::create(
            &vulkan_context.instance,
            &device_context,
            &renderer_limits,
            resource_factories.clone(),
            settings_handler.get_current(),
            device_context.physical_device_info.handle,
            &device_context.queues,
            &resource_context,
            &swapchain_context,
            resource_hub.clone(),
        )?;

        let input_handler = InputHandler::create();

        let ui_context = UiContext::new(
            &renderer_limits,
            &resource_factories.buffer_factory,
            resource_hub.image_provider.clone(),
            resource_hub.persistent_resources.clone(),
            ui_renderer,
        )?;

        let render_snapshot_handler = Arc::new(RenderSnapshotHandler::new());

        let world = World::new();
        world.add_unique(UserInputUnique::new());
        world.add_unique(WorldTimeUnique::new());
        world.add_unique(RenderViewUnique::new());
        world.add_unique(GlobalShadowUnique::new());
        world.add_unique(RenderSnapshotUnique::new(render_snapshot_handler.clone()));
        world.add_unique(ResourceResolverUnique::new(resource_hub.clone()));
        world.add_unique(ResourceLoaderUnique::new(resource_reader.clone()));
        world.add_unique(PhysicsWorldUnique::new(settings_handler.get_current()));

        info!("AmberLume created");

        Ok(Self {
            vulkan_context,
            render_surface,

            renderer_limits,
            settings_handler,

            device_context,
            swapchain_context,

            input_handler,

            ui_context,

            render_snapshot_handler,

            world,

            renderer,

            resource_context,

            providers,
            
            scene_loader: Arc::new(SceneLoader::create(resource_reader.clone())),
            
            index_managers: descriptor_index_managers,
            resource_factories,
            resource_hub,

            image_state_tracker,

            frame_counter,
        })
    }
    
    pub fn get_scene_loader(&self) -> Arc<SceneLoader> {
        self.scene_loader.clone()
    }

    pub fn on_mouse_event(&mut self, event: MouseEvent) {
        self.ui_context.on_mouse_event(event);
    }

    pub fn render(&mut self) -> Result<()> {
        let (width, height) = self.providers.surface_provider.size();
        if width == 0 || height == 0 {
            return Ok(());
        }

        if self.swapchain_context.is_out_of_date.load(Ordering::Relaxed) {
            self.invalidate_swapchain()?;
        }

        let statistics = AmberLumeStatistics {
            resources: self.resource_hub.statistics(),
            render: self.renderer.statistics(self.renderer.current_frame_index()),
            ui: self.ui_context.statistics(),
        };
        self.ui_context.render_ui(self.swapchain_context.extent, &self.settings_handler, &statistics);

        let Some(render_snapshot) = self.render_snapshot_handler.pull() else {
            warn!("Failed to pull render snapshot. Value is None");

            return Ok(());
        };
        
        self.renderer.render_frame(
            &self.device_context,
            &self.swapchain_context,
            &mut self.ui_context,
            &self.renderer_limits,
            &self.resource_context.buffer_manager,
            &self.resource_hub.resource_buffers,
            render_snapshot,
            &mut self.image_state_tracker,
        )?;

        self.resource_hub.update();

        self.frame_counter.fetch_add(1, Ordering::Relaxed);
        self.index_managers.update();

        self.settings_handler.flush();

        Ok(())
    }

    pub fn set_swapchain_out_of_date(&self) {
        self.swapchain_context.set_is_out_of_date(true);
    }

    pub fn invalidate_swapchain(&mut self) -> Result<()> {
        info!("Invalidating swapchain");

        self.device_context.queues.present_wait_idle()?;

        let new_swapchain_context = SwapchainContext::create(
            Some(&self.swapchain_context),
            &self.vulkan_context,
            &self.render_surface,
            &mut self.device_context,
            self.providers.surface_provider.clone(),
        )?;
        let new_renderer = Render::create(
            &self.vulkan_context.instance,
            &self.device_context,
            &self.renderer_limits,
            self.resource_factories.clone(),
            self.settings_handler.get_current(),
            self.device_context.physical_device_info.handle,
            &self.device_context.queues,
            &self.resource_context,
            &new_swapchain_context,
            self.resource_hub.clone(),
        )?;

        let old_swapchain_context = replace(&mut self.swapchain_context, new_swapchain_context);
        let old_renderer = replace(&mut self.renderer, new_renderer);

        old_swapchain_context.destroy(&self.device_context.device)?;
        old_renderer.destroy(&self.device_context.device, &self.resource_factories)?;

        self.image_state_tracker = ImageStateTracker::new();

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

        self.ui_context.destroy(&self.resource_factories.buffer_factory)?;

        self.renderer.destroy(&self.device_context.device, &self.resource_factories)?;

        self.resource_hub.try_unwrap()?.destroy(
            &self.index_managers,
            &self.resource_factories.managed_image_factory,
            &self.resource_factories.buffer_factory,
            &self.resource_factories.sampler_factory,
            &self.resource_factories.pipeline_layout_factory,
            &self.resource_factories.descriptor_set_layout_factory,
        )?;

        self.swapchain_context.destroy(&self.device_context.device)?;
        self.resource_context.destroy(&self.resource_factories.buffer_factory)?;

        self.resource_factories.destroy();

        self.device_context.destroy()?;

        self.render_surface.destroy(&self.vulkan_context)?;
        self.vulkan_context.destroy()?;

        info!("AmberLume destroyed");

        Ok(())
    }
}
