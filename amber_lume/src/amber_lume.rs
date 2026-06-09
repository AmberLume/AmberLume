use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use anyhow::Result;
use shipyard::World;
use tracing::{info, warn};

use raw_window_handle::RawDisplayHandle;

use crate::input_handler::hardware_key_codes::HardwareKeyCode;
use crate::input_handler::hardware_pointer_event::HardwarePointerEvent;
use crate::input_handler::input_frame::{InputFrame, PointerId};
use crate::input_handler::input_handler::InputHandler;
use crate::lifecycle::lifecycle::AmberLumeLifecycle;
use crate::limits::AmberLumeLimits;
use crate::platform_providers::io_provider::IOProvider;
use crate::platform_providers::surface_provider::SurfaceProvider;
use crate::editor::editor_state::EditorState;
use crate::render::builder::context_profile::ContextProfile;
use crate::render::device::device_context::DeviceContext;
use crate::render::device::layers::VulkanLayer;
use crate::render::device::validation_features::ValidationFeatures;
use crate::render::device::vulkan_context::VulkanContext;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::render::Render;
use crate::render::resources::resource_context::ResourceContext;
use crate::render::state::render_state::RenderState;
use crate::render::target::render_target::RenderTarget;
use crate::render::target::surface_render_target::SurfaceRenderTarget;
use crate::resources::alpaca_resource_reader::AlpacaResourceReader;
use crate::resources::binding_layout::binding_layout::BindingLayout;
use crate::resources::index_managers::IndexManagers;
use crate::resources::scene_loader::SceneLoader;
use crate::resources::skinning::bone_transform_handler::BoneTransformHandler;
use crate::resources::store::resource_store::ResourceStore;
use crate::profiler::frame_profiler::FrameProfiler;
use crate::settings::settings::EngineSettings;
use crate::settings::settings_handler::EngineSettingsHandler;
use crate::snapshot_handler::render_snapshot_handler::RenderSnapshotHandler;
use crate::statistics::amber_lume_statistics::AmberLumeStatistics;
use crate::ui::ui_context::UiContext;
use crate::ui::ui_renderer::UiRenderer;
use crate::utils::arc_utils::ArcUnwrapOrErr;
use crate::world::physics::physics_world_unique::PhysicsWorldUnique;
use crate::world::unique::global_shadow_unique::GlobalShadowUnique;
use crate::world::unique::render_snapshot_unique::RenderSnapshotUnique;
use crate::world::unique::render_view_unique::RenderViewUnique;
use crate::world::unique::resource_loader_unique::ResourceLoaderUnique;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use crate::world::unique::user_input_unique::UserInputUnique;
use crate::world::unique::world_time_unique::WorldTimeUnique;

pub struct AmberLume {
    limits: AmberLumeLimits,

    vulkan_context: Arc<VulkanContext>,
    device_context: DeviceContext,

    settings_handler: EngineSettingsHandler,
    input_handler: InputHandler,

    pub ui_context: UiContext,

    render_snapshot_handler: Arc<RenderSnapshotHandler>,

    pub world: World,

    render_state: Option<RenderState>,
    renderer: Option<Render>,

    binding_layout: Arc<BindingLayout>,

    resource_context: ResourceContext,
    index_managers: Arc<IndexManagers>,
    resource_factories: Arc<ResourceFactories>,
    resource_store: Arc<ResourceStore>,
    bone_transform_handler: Arc<BoneTransformHandler>,

    pub scene_loader: Arc<SceneLoader>,

    profiler: Arc<FrameProfiler>,

    frame_counter: Arc<AtomicU64>,
    is_paused: AtomicBool,
}

impl AmberLume {
    pub fn new(
        limits: AmberLumeLimits,
        layers: Vec<VulkanLayer>,
        validation_features: Vec<ValidationFeatures>,
        ui_renderer: Arc<dyn UiRenderer>,
        io_provider: Arc<dyn IOProvider>,
        display_handle: RawDisplayHandle,
        engine_settings: EngineSettings,
    ) -> Result<Self> {
        let settings_handler = EngineSettingsHandler::new(engine_settings);
        let frame_counter = Arc::new(AtomicU64::new(0));
        let input_handler = InputHandler::create();
        let render_snapshot_handler = Arc::new(RenderSnapshotHandler::new());

        let context_profile = ContextProfile::from(display_handle, layers, validation_features)?;
        let vulkan_context = Arc::new(VulkanContext::new(context_profile)?);
        let device_context = DeviceContext::new(&vulkan_context)?;

        let resource_reader = Arc::new(AlpacaResourceReader::new(io_provider)?);

        let resource_factories = Arc::new(ResourceFactories::create(&device_context)?);
        let resource_context = ResourceContext::create(
            &device_context.device,
            device_context.queues.clone(),
            resource_factories.clone(),
            &limits,
        )?;

        let binding_layout = Arc::new(BindingLayout::new(
            device_context.device.clone(),
            &limits.resource_limits,
            &resource_factories,
        )?);

        let resource_store = Arc::new(ResourceStore::new(
            &limits.resource_limits,
            &device_context,
            binding_layout.clone(),
            resource_reader.clone(),
            resource_context.resource_transfer.clone(),
            resource_factories.clone(),
            limits.frames_in_flight,
            frame_counter.clone(),
        )?);

        let bone_transform_handler = Arc::new(BoneTransformHandler::new(
            &resource_factories.buffer_factory,
            &limits.resource_limits,
        )?);

        let index_managers = Arc::new(IndexManagers::create(
            &limits.resource_limits,
            limits.frames_in_flight,
            frame_counter.clone(),
        ));

        let ui_context = UiContext::new(
            limits.frames_in_flight,
            &resource_factories.buffer_factory,
            resource_store.image_provider.clone(),
            resource_store.persistent_resources.clone(),
            ui_renderer,
        )?;

        let scene_loader = Arc::new(SceneLoader::create(resource_reader.clone()));

        let world = World::new();
        world.add_unique(UserInputUnique::new());
        world.add_unique(WorldTimeUnique::new());
        world.add_unique(RenderViewUnique::new());
        world.add_unique(GlobalShadowUnique::new());
        world.add_unique(RenderSnapshotUnique::new(render_snapshot_handler.clone()));
        world.add_unique(PhysicsWorldUnique::new(
            settings_handler.get_current(),
            limits.physics_limits.fixed_delta_time,
        ));
        world.add_unique(ResourceResolverUnique::new(
            resource_store.clone(),
            bone_transform_handler.clone(),
        ));
        world.add_unique(ResourceLoaderUnique::new(resource_reader));

        let render_state = Some(RenderState::new(
            &resource_factories,
            &limits,
            &index_managers,
            &binding_layout,
        )?);

        let profiler = Arc::new(FrameProfiler::new(
            &device_context,
            &resource_factories,
            limits.frames_in_flight,
            limits.profiler_limits.max_gpu_zones,
        )?);

        info!("AmberLume created");

        Ok(Self {
            limits,

            vulkan_context,
            device_context,

            settings_handler,
            input_handler,

            ui_context,

            render_snapshot_handler,

            world,

            render_state,
            renderer: None,

            binding_layout,

            resource_context,
            index_managers,
            resource_factories,
            resource_store,
            bone_transform_handler,

            scene_loader,

            profiler,

            frame_counter,
            is_paused: AtomicBool::new(false),
        })
    }

    pub fn settings_handler(&self) -> &EngineSettingsHandler {
        &self.settings_handler
    }

    pub fn handle_input(&mut self) -> InputFrame {
        let input_frame = self.input_handler.pull();
        self.ui_context.handle_input(&input_frame);

        input_frame
    }

    pub fn on_hardware_pointer_button(&mut self, id: &PointerId, event: HardwarePointerEvent) {
        self.input_handler.push_pointer_event(&id, event);
    }

    pub fn on_hardware_input(&mut self, keycode: HardwareKeyCode, pressed: bool) {
        self.input_handler.push_keycode(keycode, pressed);
    }

    pub fn create_surface_target(
        &self,
        surface_provider: Arc<dyn SurfaceProvider>,
    ) -> Result<Arc<dyn RenderTarget>> {
        let target = SurfaceRenderTarget::create(
            &self.vulkan_context,
            &self.device_context,
            surface_provider,
            self.settings_handler.get_pending().render.hdr.get(),
        )?;

        Ok(Arc::new(target))
    }

    pub fn render(&mut self) -> Result<()> {
        if self.is_paused.load(Ordering::Relaxed) {
            return Ok(());
        }

        if let Some(renderer) = self.renderer.as_ref() {
            let want_hdr = renderer.target.hdr_supported()
                && self.settings_handler.get_current().load().render.hdr.get();

            if want_hdr != renderer.target.is_hdr() {
                renderer.target.set_out_of_date(true);
            }
        }

        let needs_invalidate = self
            .renderer
            .as_ref()
            .map(|renderer| renderer.target.is_out_of_date())
            .unwrap_or(false);
        if needs_invalidate {
            self.invalidate_render_target()?;
        }

        let Some(renderer) = self.renderer.as_mut() else { return Ok(()); };

        let extent = renderer.target.extent();
        if extent.width == 0 || extent.height == 0 {
            return Ok(());
        }

        let Some(render_snapshot) = self.render_snapshot_handler.pull() else {
            warn!("Failed to pull render snapshot. Value is None");

            return Ok(());
        };

        renderer.render_frame(
            &self.device_context,
            &mut self.ui_context,
            &self.limits,
            &self.resource_store.resource_buffers,
            render_snapshot,
        )?;

        self.resource_store.update();
        self.frame_counter.fetch_add(1, Ordering::Relaxed);
        self.index_managers.update();
        self.settings_handler.flush();

        Ok(())
    }

    pub fn set_render_target_out_of_date(&self) {
        if let Some(renderer) = self.renderer.as_ref() {
            renderer.target.set_out_of_date(true);
        }
    }

    fn invalidate_render_target(&mut self) -> Result<()> {
        info!("Invalidating render target");

        let Some(old_renderer) = self.renderer.take() else { return Ok(()); };

        let new_renderer = old_renderer.invalidate(
            &self.vulkan_context.instance,
            &self.vulkan_context,
            &self.device_context,
            &self.limits,
            self.resource_factories.clone(),
            self.settings_handler.get_current(),
            self.device_context.physical_device_info.handle,
            &self.resource_context,
            self.binding_layout.clone(),
            self.bone_transform_handler.clone(),
            self.resource_store.clone(),
        )?;

        self.renderer = Some(new_renderer);

        info!("Render target invalidated");

        Ok(())
    }

    pub fn render_ui(&mut self, input_frame: &InputFrame) {
        let Some(renderer) = self.renderer.as_ref() else { return; };

        let statistics = AmberLumeStatistics {
            frame_profile: self.profiler.last_profile(),
            resources: self.resource_store.statistics(),
            render: renderer.statistics(),
            ui: self.ui_context.statistics(),
        };

        let editor_state = EditorState {
            picked_entity: renderer.picked_entity(),
        };

        self.ui_context.render_ui(
            renderer.target.extent(),
            input_frame,
            &self.settings_handler,
            &statistics,
            &editor_state,
        );
    }

    pub fn statistics(&self) -> Option<AmberLumeStatistics> {
        let renderer = self.renderer.as_ref()?;

        Some(AmberLumeStatistics {
            frame_profile: self.profiler.last_profile(),
            resources: self.resource_store.statistics(),
            render: renderer.statistics(),
            ui: self.ui_context.statistics(),
        })
    }

    pub fn destroy(mut self) -> Result<()> {
        self.device_context.queues.all_wait_idle()?;

        if let Some(renderer) = self.renderer.take() {
            let render_state = renderer.destroy(
                &self.vulkan_context,
                &self.device_context,
                &self.resource_factories,
            )?;
            self.render_state = Some(render_state);
        }

        self.world.clear();
        let _ = self.world.remove_unique::<ResourceResolverUnique>();
        let _ = self.world.remove_unique::<ResourceLoaderUnique>();

        if let Some(render_state) = self.render_state {
            render_state.destroy(&self.resource_factories, &self.index_managers)?;
        }

        self.ui_context.destroy(&self.resource_factories.buffer_factory)?;

        self.resource_store.try_unwrap()?.destroy(&self.resource_factories)?;
        self.bone_transform_handler.try_unwrap()?.destroy(&self.resource_factories.buffer_factory)?;
        self.binding_layout.try_unwrap()?.destroy(&self.resource_factories)?;
        self.resource_context.destroy(&self.resource_factories.buffer_factory)?;
        self.profiler.try_unwrap()?.destroy(&self.resource_factories)?;
        self.resource_factories.destroy();

        self.device_context.destroy()?;
        self.vulkan_context.destroy()?;

        info!("AmberLume destroyed gracefully");

        Ok(())
    }
}

impl AmberLumeLifecycle for AmberLume {
    fn attach_render_target(&mut self, target: Arc<dyn RenderTarget>) -> Result<()> {
        if self.renderer.is_some() {
            self.detach_render_target()?;
        }

        let renderer = Render::create(
            &self.vulkan_context.instance,
            &self.device_context,
            &self.limits,
            target,
            self.resource_factories.clone(),
            self.settings_handler.get_current(),
            self.device_context.physical_device_info.handle,
            &self.device_context.queues,
            &self.resource_context,
            self.resource_store.clone(),
            self.binding_layout.clone(),
            self.bone_transform_handler.clone(),
            self.profiler.clone(),
            self.render_state.take().unwrap(),
        )?;

        self.renderer = Some(renderer);

        info!("AmberLume render target attached");

        Ok(())
    }

    fn detach_render_target(&mut self) -> Result<()> {
        let Some(renderer) = self.renderer.take() else { return Ok(()); };

        self.device_context.queues.all_wait_idle()?;

        let render_state = renderer.destroy(
            &self.vulkan_context,
            &self.device_context,
            &self.resource_factories,
        )?;
        self.render_state = Some(render_state);

        info!("AmberLume render target detached");

        Ok(())
    }

    fn is_render_target_attached(&self) -> bool {
        self.renderer.is_some()
    }

    fn pause(&mut self) {
        self.is_paused.store(true, Ordering::Relaxed);
        info!("AmberLume paused");
    }

    fn resume(&mut self) {
        self.is_paused.store(false, Ordering::Relaxed);
        info!("AmberLume resumed");
    }

    fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::Relaxed)
    }
}
