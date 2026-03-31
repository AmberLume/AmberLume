use crate::render::device::device_context::DeviceContext;
use crate::render::render_pass::depth::depth_render_pass::DepthRenderPass;
use crate::render::render_pass::main::main_render_pass::MainRenderPass;
use crate::render::render_pass::render_pass_context::RenderPassContext;
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
use crate::render::statistics::cpu_render_statistics::{CpuRenderStatistics, CpuRenderStatisticsSnapshot};
use crate::render::statistics::gpu_render_statistics::GpuRenderStatistics;
use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::queue::queues::Queues;
use crate::render::render_pass::culling_indirect::culling_indirect_render_pass::CullingIndirectRenderPass;
use crate::render::render_pass::render_pass_layout::{RenderView, RenderViewsLayout};
use crate::render::render_pass::shadow_mask::shadow_mask_render_pass::ShadowMaskRenderPass;
use crate::render::shadows::shadow_cascades_helper::ShadowCascadeHelper;
use crate::render::render_pass::shadows::shadows_render_pass::ShadowsRenderPass;
use crate::render::resources::resource_context::ResourceContext;
use crate::render::render_pass::ui::ui_render_pass::UiRenderPass;
use crate::render::frame::frame_context::FrameContext;
use crate::render::pass_registry::PassRegistry;
use crate::render::render_pass::frame_data_context::FrameDataContext;
use crate::render::render_pass::physics_debug::physics_debug_render_pass::PhysicsDebugRenderPass;
use crate::render::statistics::raw::gpu_render_stats_handler::RawGpuRenderStatsHandler;
use crate::render::statistics::raw::gpu_stage_measurement_recorder::GpuMeasurementStages;
use crate::resources::descriptor_set_manager::DescriptorSetManager;
use crate::resources::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::settings::settings::EngineSettings;
use crate::statistics::measurement::MeasurementInstant;
use crate::statistics::statistics_context::StatisticsContext;
use crate::ui::ui_context::UiContext;
use crate::utils::matrix_wrappers::ViewProjectionMatrix;

pub struct Render {
    render_context: RenderContext,

    pass_registry: PassRegistry,

    descriptor_set_manager: Arc<DescriptorSetManager>,
    pipeline_layout_registry: Arc<PipelineLayoutRegistry>,

    render_statistics_handler: RawGpuRenderStatsHandler,

    cpu_render_statistics: Arc<CpuRenderStatistics>,
    gpu_render_statistics: Arc<GpuRenderStatistics>,
}

impl Render {
    pub fn create(
        instance: &Instance,
        device: &Device,
        renderer_limits: &RendererLimits,
        settings: Arc<ArcSwap<EngineSettings>>,
        physical_device: PhysicalDevice,
        queues: &Queues,
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        statistics_context: &StatisticsContext,
        resource_hub: Arc<ResourceHub>,
    ) -> Result<Self> {
        let render_context = RenderContext::create(
            &instance,
            &device,
            &renderer_limits,
            &resource_hub,
            physical_device,
            queues,
            &swapchain_context,
        )?;

        let render_statistics_handler = RawGpuRenderStatsHandler::create(
            device.clone(),
            resource_context.buffer_manager.clone(),
            renderer_limits.frames_in_flight,
        )?;

        let pipeline_provider = resource_hub.get_pipeline_provider();
        let compute_pipeline_provider = resource_hub.get_compute_pipeline_provider();

        let culling_indirect_render_pass = CullingIndirectRenderPass::create(
            &resource_context,
            &compute_pipeline_provider,
            &resource_hub.pipeline_layout_registry,
        )?;
        let depth_render_pass = DepthRenderPass::create(
            &resource_context,
            &render_context,
            &pipeline_provider,
            &resource_hub.pipeline_layout_registry,
        )?;
        let shadow_mask_render_pass = ShadowMaskRenderPass::create(
            &resource_context,
            &pipeline_provider,
            &resource_hub.pipeline_layout_registry,
            resource_hub.persistent_resources.clone(),
        )?;
        let shadows_render_pass = ShadowsRenderPass::create(
            &resource_context,
            &pipeline_provider,
            &resource_hub.pipeline_layout_registry,
            resource_hub.persistent_resources.clone(),
        )?;
        let main_render_pass = MainRenderPass::create(
            &resource_context,
            &swapchain_context,
            &render_context,
            &pipeline_provider,
            &resource_hub.pipeline_layout_registry,
        )?;
        let physics_debug_render_pass = PhysicsDebugRenderPass::create(
            &resource_context,
            &swapchain_context,
            &render_context,
            &pipeline_provider,
            &resource_hub.pipeline_layout_registry,
            settings,
        )?;
        let ui_render_pass = UiRenderPass::create(
            &resource_context,
            &swapchain_context,
            &pipeline_provider,
            &resource_hub.pipeline_layout_registry,
        )?;

        let pass_registry = PassRegistry::create(
            culling_indirect_render_pass,
            depth_render_pass,
            shadows_render_pass,
            shadow_mask_render_pass,
            main_render_pass,
            physics_debug_render_pass,
            ui_render_pass,
        );

        Ok(Self {
            render_context,

            pass_registry,

            descriptor_set_manager: resource_hub.descriptor_set_manager.clone(),
            pipeline_layout_registry: resource_hub.pipeline_layout_registry.clone(),

            render_statistics_handler,

            cpu_render_statistics: statistics_context.cpu_render.clone(),
            gpu_render_statistics: statistics_context.gpu_render.clone(),
        })
    }

    pub fn render_frame(
        &mut self,
        device_context: &DeviceContext,
        swapchain_context: &SwapchainContext,
        ui_context: &mut UiContext,
        renderer_limits: &RendererLimits,
        buffer_manager: &BufferManager,
        render_snapshot: Arc<RenderSnapshot>,
    ) -> Result<()> {
        let frame_index = self.render_context.next_frame_index();
        let frame_context = &mut self.render_context.get_frame(frame_index)?;

        unsafe { device_context.device.wait_for_fences(&[frame_context.fence], true, u64::MAX)? };

        let gpu_render_statistics = self.render_statistics_handler.read(frame_index)?;

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

        let ui_build = MeasurementInstant::start();
        let ui_snapshot = ui_context.build_ui_snapshot()?;
        let ui_build = ui_build.capture();

        let render_views_layout = self.build_render_views_layout(&swapchain_context, &renderer_limits, &render_snapshot);
        let frame_data_context = FrameDataContext::create(
            &renderer_limits,
            &render_views_layout,
            render_snapshot.clone(),
            ui_snapshot,
        );

        let render_pass_context = RenderPassContext::create(
            &device_context,
            &swapchain_context,
            &self.render_context,
            &renderer_limits,
            &frame_context.command_recording,
            image_index,
            frame_index,
            &render_views_layout,
            &buffer_manager,
        )?;

        let render_commands = MeasurementInstant::start();
        self.collect_render_commands(&frame_data_context, &render_pass_context, frame_index, frame_context)?;
        let render_commands = render_commands.capture();

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

        self.cpu_render_statistics.push(CpuRenderStatisticsSnapshot {
            ui_build,
            render_commands,
        });
        self.gpu_render_statistics.fill(device_context, gpu_render_statistics);

        Ok(())
    }

    fn collect_render_commands(
        &self,
        frame_data_context: &FrameDataContext,
        render_pass_context: &RenderPassContext,
        frame_index: FrameIndex,
        frame_context: &FrameContext,
    ) -> Result<()> {
        render_pass_context.begin_command_recording()?;
        self.render_statistics_handler.reset(frame_context.command_recording.command_buffer, frame_index);

        self.render_statistics_handler.stage_recorder.record(
            frame_context.command_recording.command_buffer,
            PipelineStageFlags::TOP_OF_PIPE,
            frame_index,
            GpuMeasurementStages::PipelineStart,
        );

        self.descriptor_set_manager.bind(
            render_pass_context.command_recording.command_buffer,
            self.pipeline_layout_registry.get(PipelineLayoutType::General),
        );

        self.pass_registry.run_each(&frame_data_context, &render_pass_context)?;

        render_pass_context.finalize();
        self.render_statistics_handler.stage_recorder.record(
            frame_context.command_recording.command_buffer,
            PipelineStageFlags::BOTTOM_OF_PIPE,
            frame_index,
            GpuMeasurementStages::PipelineEnd,
        );

        self.render_statistics_handler.collect(
            frame_context.command_recording.command_buffer,
            frame_index,
        );
        render_pass_context.end_command_recording()?;

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

    pub fn destroy(self, device: &Device) -> Result<()> {
        self.render_statistics_handler.destroy()?;

        self.pass_registry.destroy()?;

        self.render_context.destroy(&device)?;

        Ok(())
    }
}
