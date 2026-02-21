use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::render_pass::depth::depth_render_pass::DepthRenderPass;
use crate::render::vulkan::render_pass::main::main_render_pass::MainRenderPass;
use crate::render::vulkan::render_pass::render_pass::RenderPass;
use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use crate::render::vulkan::renderer::render_context::RenderContext;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use crate::resources::resource_hub::ResourceHub;
use crate::snapshot_handler::world_snapshot::WorldSnapshot;
use anyhow::{bail, Result};
use ash::{vk, Device, Instance};
use ash::vk::{Fence, PhysicalDevice, PipelineStageFlags, PresentInfoKHR, SubmitInfo};
use std::slice;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::vulkan::queue::queues::Queues;
use crate::render::vulkan::render_pass::collider::collider_render_pass::ColliderRenderPass;
use crate::render::vulkan::render_pass::collider_culling_pass::collider_culling_render_pass::ColliderCullingRenderPass;
use crate::render::vulkan::render_pass::culling_indirect_pass::culling_indirect_render_pass::CullingIndirectRenderPass;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::render::vulkan::render_pass::ui_render_pass::ui_render_pass::UiRenderPass;
use crate::render::vulkan::renderer::frame_context::FrameContext;
use crate::render::vulkan::renderer::stats::frame_stats::FrameStats;
use crate::render::vulkan::renderer::stats::gpu_render_stats_handler::GpuRenderStatsHandler;
use crate::render::vulkan::renderer::stats::gpu_stage_measurement_recorder::GpuMeasurementStages;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::resources::resource_factories::ResourceFactories;
use crate::ui::ui_context::UiContext;
use crate::system_stats::SystemStatsHolder;

pub struct Renderer {
    render_context: RenderContext,

    renderer_limits: RendererLimits,

    persistent_resources: Arc<PersistentResources>,

    render_passes: Vec<Box<dyn RenderPass>>,

    render_stats_handler: GpuRenderStatsHandler,
}

impl Renderer {
    pub fn create(
        instance: &Instance,
        device: &Device,
        renderer_limits: RendererLimits,
        physical_device: PhysicalDevice,
        queues: &Queues,
        resource_factories: &ResourceFactories,
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        resource_hub: Arc<ResourceHub>,
        persistent_resources: Arc<PersistentResources>,
    ) -> Result<Self> {
        let render_context = RenderContext::create(
            &instance,
            &device,
            &resource_factories,
            physical_device,
            queues,
            &swapchain_context,
        )?;

        let render_stats_reader = GpuRenderStatsHandler::create(
            device.clone(),
            &resource_factories.managed_buffer_factory,
            swapchain_context.swapchain_images.len() as u32,
        )?;

        let pipeline_provider = resource_hub.get_pipeline_provider();
        let compute_pipeline_provider = resource_hub.get_compute_pipeline_provider();

        let culling_indirect_render_pass = CullingIndirectRenderPass::create(
            &resource_context,
            &compute_pipeline_provider,
            &persistent_resources,
            &render_stats_reader,
        )?;
        let collider_culling_render_pass = ColliderCullingRenderPass::create(
            &resource_context,
            &compute_pipeline_provider,
            &persistent_resources,
        )?;
        let depth_render_pass = DepthRenderPass::create(
            &resource_context,
            &render_context,
            &pipeline_provider,
            &persistent_resources,
        )?;
        let main_render_pass = MainRenderPass::create(
            &resource_context,
            &swapchain_context,
            &render_context,
            &pipeline_provider,
            &persistent_resources,
        )?;
        let collider_render_pass = ColliderRenderPass::create(
            &resource_context,
            &swapchain_context,
            &render_context,
            &pipeline_provider,
            &persistent_resources,
        )?;
        let ui_render_pass = UiRenderPass::create(
            &resource_context,
            &swapchain_context,
            &pipeline_provider,
            &persistent_resources,
        )?;

        let render_passes: Vec<Box<dyn RenderPass>> = vec![
            Box::new(culling_indirect_render_pass),
            Box::new(collider_culling_render_pass),
            Box::new(depth_render_pass),
            Box::new(main_render_pass),
            Box::new(collider_render_pass),
            Box::new(ui_render_pass),
        ];

        Ok(Self {
            render_context,

            renderer_limits,

            persistent_resources,

            render_passes,

            render_stats_handler: render_stats_reader,
        })
    }

    pub fn render_frame(
        &mut self,
        device_context: &DeviceContext,
        swapchain_context: &SwapchainContext,
        ui_context: &mut UiContext,
        system_stats_handler: &mut SystemStatsHolder,
        world_snapshot: Arc<WorldSnapshot>,
        current_frame: u64,
    ) -> Result<()> {
        let total_frame_time_instant = Instant::now();

        let frame_index = self.render_context.next_frame_index();
        let frame_context = &mut self.render_context.get_frame(frame_index)?;

        unsafe { device_context.device.wait_for_fences(&[frame_context.fence], true, u64::MAX)? };

        let gpu_render_stats = self.render_stats_handler.read(frame_index)?;

        let (image_index, suboptimal) = self.acquire_image(swapchain_context, frame_context)?;

        let cpu_frame_time_instant = Instant::now();

        let ui_snapshot = ui_context.build_ui_snapshot(current_frame)?;

        let render_pass_context = RenderPassContext::create(
            &device_context,
            &swapchain_context,
            &self.render_context,
            &self.renderer_limits,
            &frame_context.command_recording,
            image_index,
            frame_index as u32,
            world_snapshot.clone(),
            ui_snapshot,
        )?;

        self.collect_render_commands(&render_pass_context, frame_index as u32, frame_context)?;

        let cpu_data_prepare_time = cpu_frame_time_instant.elapsed().as_secs_f32();
        
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
            info!("Swapchain swapchain image out of date");

            swapchain_context.set_is_out_of_date(true);
        }

        let total_frame_time = total_frame_time_instant.elapsed().as_secs_f32();

        let gpu_render_time = {
            let ticks_delta = gpu_render_stats.render_time.pipeline_end - gpu_render_stats.render_time.pipeline_start;

            let nanos_delta = ticks_delta as f64 * device_context.physical_device_info.timestamp_period as f64;

            (nanos_delta / 1_000_000_000.0) as f32
        };
        system_stats_handler.register_submesh_rendered(gpu_render_stats.submeshes_rendered);
        system_stats_handler.register_submesh_culled(gpu_render_stats.submeshes_culled);
        system_stats_handler.register_frame_stats(FrameStats {
            cpu_data_prepare_time,
            gpu_render_time,
            total_frame_time,
        });

        Ok(())
    }

    fn acquire_image(
        &self,
        swapchain_context: &SwapchainContext,
        frame_context: &FrameContext,
    ) -> Result<(u32, bool)> {
        match unsafe {
            swapchain_context.loader.acquire_next_image(
                swapchain_context.handle,
                u64::MAX,
                frame_context.acquire_semaphore,
                Fence::null(),
            )
        } {
            Ok(result) => Ok(result),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                info!("Swapchain image out of date");

                swapchain_context.set_is_out_of_date(true);

                bail!("Swapchain image out of date");
            }
            Err(e) => Err(e.into()),
        }
    }

    fn collect_render_commands(
        &self,
        render_pass_context: &RenderPassContext,
        frame_index: u32,
        frame_context: &FrameContext,
    ) -> Result<()> {
        render_pass_context.begin_command_recording()?;
        self.render_stats_handler.reset(frame_context.command_recording.command_buffer, frame_index);

        self.render_stats_handler.stage_recorder.record(
            frame_context.command_recording.command_buffer,
            PipelineStageFlags::TOP_OF_PIPE,
            frame_index,
            GpuMeasurementStages::PipelineStart,
        );

        self.persistent_resources.descriptor_sets.global.bind(
            render_pass_context.command_recording.command_buffer,
            self.persistent_resources.pipeline_layouts.global,
        );

        for render_pass in &self.render_passes {
            let is_enabled = render_pass.is_enabled();

            if is_enabled {
                render_pass.begin_record_commands(&render_pass_context)?;
                render_pass.record_commands(&render_pass_context)?;
                render_pass.end_record_commands(&render_pass_context)?;
            }
        }

        render_pass_context.finalize();
        self.render_stats_handler.stage_recorder.record(
            frame_context.command_recording.command_buffer,
            PipelineStageFlags::BOTTOM_OF_PIPE,
            frame_index,
            GpuMeasurementStages::PipelineEnd,
        );

        self.render_stats_handler.collect(
            frame_context.command_recording.command_buffer,
            frame_index,
        );
        render_pass_context.end_command_recording()?;

        Ok(())
    }

    pub fn destroy(
        mut self,
        device: &Device,
        resource_factories: &ResourceFactories,
    ) -> Result<()> {
        self.render_stats_handler.destroy(&resource_factories.managed_buffer_factory)?;

        for render_pass in &self.render_passes {
            render_pass.destroy()?;
        }
        self.render_passes.clear();

        self.render_context.destroy(&device, &resource_factories)?;

        Ok(())
    }
}
