use crate::render::device::device_context::DeviceContext;
use crate::render::pass::depth::depth_render_pass::DepthPass;
use crate::render::pass::main::main_render_pass::MainPass;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_context::RenderContext;
use crate::render::swapchain::swapchain_context::SwapchainContext;
use crate::resources::resource_hub::ResourceHub;
use crate::snapshot_handler::render_snapshot::RenderSnapshot;
use anyhow::{bail, Result};
use ash::{vk, Device, Instance};
use ash::vk::{Fence, PhysicalDevice, PipelineStageFlags, PresentInfoKHR, SubmitInfo};
use std::slice;
use std::sync::Arc;
use arc_swap::ArcSwap;
use tracing::info;
use crate::ids::FrameIndex;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::queue::queues::Queues;
use crate::render::pass::culling_indirect::culling_indirect_pass::CullingIndirectPass;
use crate::render::pass::pass_layout::{RenderView, RenderViewsLayout};
use crate::render::pass::shadow_mask::shadow_mask_render_pass::ShadowMaskPass;
use crate::render::shadows::shadow_cascades_helper::ShadowCascadeHelper;
use crate::render::pass::shadows::shadows_render_pass::ShadowsPass;
use crate::render::resources::resource_context::ResourceContext;
use crate::render::pass::ui::ui_render_pass::UiPass;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass::Pass;
use crate::render::pass::physics_debug::physics_debug_render_pass::PhysicsDebugPass;
use crate::render::render_graph::image_state_tracker::image_state_tracker::ImageStateTracker;
use crate::render::render_graph::pass_graph::PassGraph;
use crate::render::renderer_statistics::{RenderStatistics, RenderStatisticsMeasurement};
use crate::render::statistics::interval::gpu_interval_measurement::GpuIntervalMeasurement;
use crate::render::statistics::pass_profiler::PassProfiler;
use crate::resources::descriptor_set_manager::DescriptorSetManager;
use crate::resources::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::resources::resource_buffers::ResourceBuffers;
use crate::settings::settings::EngineSettings;
use crate::ui::ui_context::UiContext;
use crate::utils::matrix_wrappers::ViewProjectionMatrix;

pub struct Render {
    render_context: RenderContext,

    pass_graph: PassGraph,
    pass_profiler: PassProfiler,

    descriptor_set_manager: Arc<DescriptorSetManager>,
    pipeline_layout_registry: Arc<PipelineLayoutRegistry>,

    statistics: RenderStatisticsMeasurement,
    total_dispatch_measurement: GpuIntervalMeasurement,
}

impl Render {
    pub fn create(
        instance: &Instance,
        device_context: &DeviceContext,
        renderer_limits: &RendererLimits,
        resource_factories: &ResourceFactories,
        settings: Arc<ArcSwap<EngineSettings>>,
        physical_device: PhysicalDevice,
        queues: &Queues,
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        resource_hub: Arc<ResourceHub>,
    ) -> Result<Self> {
        let render_context = RenderContext::create(
            &instance,
            &device_context.device,
            &renderer_limits,
            &resource_hub,
            physical_device,
            queues,
            &swapchain_context,
        )?;

        let culling_indirect_render_pass = CullingIndirectPass::create(
            &resource_context,
            &renderer_limits,
            &resource_factories,
            &resource_hub.compute_pipeline_provider,
            &resource_hub.pipeline_layout_registry,
        )?;
        let depth_render_pass = DepthPass::create(
            &resource_context,
            &render_context,
            &resource_hub.pipeline_provider,
            &resource_hub.pipeline_layout_registry,
        )?;
        let shadow_mask_render_pass = ShadowMaskPass::create(
            &resource_context,
            &resource_hub.pipeline_provider,
            &resource_hub.pipeline_layout_registry,
            resource_hub.persistent_resources.clone(),
        )?;
        let shadows_render_pass = ShadowsPass::create(
            &resource_context,
            &resource_hub.pipeline_provider,
            &resource_hub.pipeline_layout_registry,
            resource_hub.persistent_resources.clone(),
        )?;
        let main_render_pass = MainPass::create(
            &resource_context,
            &swapchain_context,
            &render_context,
            &resource_hub.pipeline_provider,
            &resource_hub.pipeline_layout_registry,
        )?;
        let physics_debug_render_pass = PhysicsDebugPass::create(
            &resource_context,
            &swapchain_context,
            &render_context,
            &resource_hub.pipeline_provider,
            &resource_hub.pipeline_layout_registry,
            settings,
        )?;
        let ui_render_pass = UiPass::create(
            &swapchain_context,
            &resource_hub.pipeline_provider,
            &resource_hub.pipeline_layout_registry,
        )?;

        let mut pass_profiler = PassProfiler::new();
        pass_profiler.register(culling_indirect_render_pass.name(), &device_context, &resource_factories, renderer_limits.frames_in_flight)?;
        pass_profiler.register(depth_render_pass.name(), &device_context, &resource_factories, renderer_limits.frames_in_flight)?;
        pass_profiler.register(shadows_render_pass.name(), &device_context, &resource_factories, renderer_limits.frames_in_flight)?;
        pass_profiler.register(shadow_mask_render_pass.name(), &device_context, &resource_factories, renderer_limits.frames_in_flight)?;
        pass_profiler.register(main_render_pass.name(), &device_context, &resource_factories, renderer_limits.frames_in_flight)?;
        pass_profiler.register(physics_debug_render_pass.name(), &device_context, &resource_factories, renderer_limits.frames_in_flight)?;
        pass_profiler.register(ui_render_pass.name(), &device_context, &resource_factories, renderer_limits.frames_in_flight)?;

        let mut pass_graph = PassGraph::new();
        pass_graph.add_pass(culling_indirect_render_pass);
        pass_graph.add_pass(depth_render_pass);
        pass_graph.add_pass(shadows_render_pass);
        pass_graph.add_pass(shadow_mask_render_pass);
        pass_graph.add_pass(main_render_pass);
        pass_graph.add_pass(physics_debug_render_pass);
        pass_graph.add_pass(ui_render_pass);

        let total_dispatch_measurement = GpuIntervalMeasurement::new(
            &device_context,
            "total_dispatch",
            &resource_factories.query_pool_factory,
            &resource_factories.buffer_factory,
            renderer_limits.frames_in_flight,
        )?;

        Ok(Self {
            render_context,

            pass_graph,
            pass_profiler,

            descriptor_set_manager: resource_hub.descriptor_set_manager.clone(),
            pipeline_layout_registry: resource_hub.pipeline_layout_registry.clone(),

            statistics: RenderStatisticsMeasurement::new(),
            total_dispatch_measurement,
        })
    }

    pub fn current_frame_index(&self) -> FrameIndex {
        self.render_context.current_frame_index()
    }
    
    pub fn render_frame(
        &mut self,
        device_context: &DeviceContext,
        swapchain_context: &SwapchainContext,
        ui_context: &mut UiContext,
        renderer_limits: &RendererLimits,
        buffer_manager: &BufferManager,
        resource_buffers: &ResourceBuffers,
        render_snapshot: Arc<RenderSnapshot>,
        image_state_tracker: &mut ImageStateTracker,
    ) -> Result<()> {
        let frame_index = self.render_context.next_frame_index();
        let frame_context = self.render_context.get_frame(frame_index)?;

        unsafe { device_context.device.wait_for_fences(&[frame_context.fence], true, u64::MAX)? };

        let (image_index, suboptimal) = match unsafe {
            swapchain_context.loader.acquire_next_image(
                swapchain_context.handle,
                u64::MAX,
                frame_context.acquire_semaphore,
                Fence::null(),
            )
        } {
            Ok(result) => result,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                swapchain_context.set_is_out_of_date(true);

                info!("Swapchain image out of date");

                return Ok(());
            }
            Err(error) => bail!(error),
        };

        self.statistics.total_time.start();
        let ui_snapshot = ui_context.build_ui_snapshot()?;
        self.statistics.total_time.finish();

        let render_views_layout = self.build_render_views_layout(&swapchain_context, &renderer_limits, &render_snapshot);
        let frame_data_context = FrameDataContext::create(
            frame_index,
            &device_context,
            &frame_context.command_recording,
            &renderer_limits,
            &render_views_layout,
            render_snapshot.clone(),
            ui_snapshot,
            &ui_context
        );

        let render_pass_context = PassContext::create(
            &device_context,
            &swapchain_context,
            &self.render_context,
            &renderer_limits,
            &frame_context.command_recording,
            image_index,
            frame_index,
            &render_views_layout,
            &buffer_manager,
            &resource_buffers,
        )?;

        self.statistics.collect_record_commands.start();
        Self::collect_render_commands(
            &frame_data_context,
            &render_pass_context,
            &self.descriptor_set_manager,
            &self.pipeline_layout_registry,
            &self.total_dispatch_measurement,
            &mut self.pass_graph,
            image_state_tracker,
            &mut self.pass_profiler,
        )?;
        self.statistics.collect_record_commands.finish();

        let present_semaphore = self.render_context.get_present_semaphore(image_index)?;

        let wait_semaphores = [frame_context.acquire_semaphore];
        let signal_semaphores = [present_semaphore];
        let wait_stages = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit_info = SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(slice::from_ref(
                &frame_context.command_recording.command_buffer,
            ))
            .signal_semaphores(&signal_semaphores);

        unsafe { device_context.device.reset_fences(&[frame_context.fence])? };

        device_context.queues.submit_graphics(submit_info, frame_context.fence)?;

        let wait_semaphores = [present_semaphore];
        let swapchains = [swapchain_context.handle];
        let image_indices = [image_index];
        let present_info = PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let present_result = device_context.queues.present(&swapchain_context, present_info);

        let is_surface_out_of_date = matches!(
            present_result,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::ERROR_SURFACE_LOST_KHR)
        );

        if suboptimal || is_surface_out_of_date || present_result == Ok(true) {
            info!("Swapchain image out of date");

            swapchain_context.set_is_out_of_date(true);
        }

        Ok(())
    }

    fn collect_render_commands(
        frame_data_context: &FrameDataContext,
        pass_context: &PassContext,
        descriptor_set_manager: &DescriptorSetManager,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        total_dispatch_measurement: &GpuIntervalMeasurement,
        pass_graph: &mut PassGraph,
        image_state_tracker: &mut ImageStateTracker,
        pass_profiler: &mut PassProfiler,
    ) -> Result<()> {
        pass_context.begin_command_recording()?;
        total_dispatch_measurement.record_start(
            pass_context.command_recording.command_buffer,
            pass_context.frame_index,
            0,
        );

        image_state_tracker.begin_frame();

        descriptor_set_manager.bind(
            pass_context.command_recording.command_buffer,
            pipeline_layout_registry.get(PipelineLayoutType::General),
        );

        pass_graph.run(&frame_data_context, &pass_context, image_state_tracker, pass_profiler)?;

        pass_context.finalize(image_state_tracker);

        total_dispatch_measurement.record_end(
            pass_context.command_recording.command_buffer,
            pass_context.frame_index,
            0,
        );
        pass_context.end_command_recording()?;

        Ok(())
    }

    fn build_render_views_layout(
        &self,
        swapchain_context: &SwapchainContext,
        renderer_limits: &RendererLimits,
        render_snapshot: &RenderSnapshot,
    ) -> RenderViewsLayout {
        let extent = swapchain_context.extent;
        let aspect_ratio = extent.width as f32 / extent.height as f32;

        let camera_view = render_snapshot.camera.view();
        let camera_projection = render_snapshot.camera.projection(aspect_ratio);

        let global_shadow_cascades = ShadowCascadeHelper::from_camera_projection(
            &camera_view,
            &camera_projection,
            &renderer_limits.shadow_map_limits.global_cascades,
            renderer_limits.shadow_map_limits.resolution,
            render_snapshot.global_shadows_direction,
            render_snapshot.camera.near,
            render_snapshot.camera.far,
            10.0,
        );

        RenderViewsLayout {
            main: RenderView {
                view_projection: ViewProjectionMatrix::from_view_projection(&camera_view, &camera_projection).vulkan_corrected(),
            },
            global_shadow_cascades: global_shadow_cascades.into_iter().map(|projection| {
                RenderView {
                    view_projection: projection,
                }
            }).collect(),
        }
    }

    pub fn statistics(&self, frame_index: FrameIndex) -> RenderStatistics {
        RenderStatistics {
            total_time: self.statistics.total_time.collect(),
            collect_record_commands: self.statistics.collect_record_commands.collect(),

            total_dispatch: self.total_dispatch_measurement.collect(frame_index),
            
            pass_profiles: self.pass_profiler.collect(frame_index),
        }
    }

    pub fn destroy(self, device: &Device, resource_factories: &ResourceFactories) -> Result<()> {
        self.total_dispatch_measurement.destroy(&resource_factories.buffer_factory)?;

        self.pass_profiler.destroy(&resource_factories)?;
        self.pass_graph.destroy(&resource_factories)?;

        self.render_context.destroy(&device)?;

        Ok(())
    }
}
