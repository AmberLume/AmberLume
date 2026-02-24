use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::render_pass::depth::depth_render_pass::DepthRenderPass;
use crate::render::vulkan::render_pass::main::main_render_pass::MainRenderPass;
use crate::render::vulkan::render_pass::render_pass::RenderPass;
use crate::render::vulkan::render_pass::render_pass_context::{RenderPassContext, RenderView, RenderViewsLayout};
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
use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};
use tracing::info;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::vulkan::queue::queues::Queues;
use crate::render::vulkan::render_pass::culling_indirect_pass::culling_indirect_render_pass::CullingIndirectRenderPass;
use crate::render::vulkan::render_pass::shadow_mask::shadow_mask_render_pass::ShadowMaskRenderPass;
use crate::render::vulkan::render_pass::shadows::shadows_render_pass::ShadowsRenderPass;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::render::vulkan::render_pass::ui_render_pass::ui_render_pass::UiRenderPass;
use crate::render::vulkan::renderer::frame_context::FrameContext;
use crate::render::vulkan::renderer::stats::frame_stats::FrameStats;
use crate::render::vulkan::renderer::stats::gpu_render_stats_handler::GpuRenderStatsHandler;
use crate::render::vulkan::renderer::stats::gpu_stage_measurement_recorder::GpuMeasurementStages;
use crate::resources::descriptor_index_managers::IndexManagers;
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
        index_managers: &IndexManagers,
        resource_factories: &ResourceFactories,
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        resource_hub: Arc<ResourceHub>,
        persistent_resources: Arc<PersistentResources>,
    ) -> Result<Self> {
        let render_context = RenderContext::create(
            &instance,
            &device,
            &index_managers,
            &persistent_resources,
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
        let depth_render_pass = DepthRenderPass::create(
            &resource_context,
            &render_context,
            &pipeline_provider,
            &persistent_resources,
        )?;
        let shadow_mask_render_pass = ShadowMaskRenderPass::create(
            &pipeline_provider,
            persistent_resources.clone(),
        )?;
        let shadows_render_pass = ShadowsRenderPass::create(
            &resource_context,
            &pipeline_provider,
            persistent_resources.clone(),
        )?;
        let main_render_pass = MainRenderPass::create(
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
            Box::new(depth_render_pass),
            Box::new(shadows_render_pass),
            Box::new(shadow_mask_render_pass),
            Box::new(main_render_pass),
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
    ) -> Result<()> {
        let total_frame_time_instant = Instant::now();

        let frame_index = self.render_context.next_frame_index();
        let frame_context = &mut self.render_context.get_frame(frame_index)?;

        unsafe { device_context.device.wait_for_fences(&[frame_context.fence], true, u64::MAX)? };

        let gpu_render_stats = self.render_stats_handler.read(frame_index)?;

        let (image_index, suboptimal) = self.acquire_image(swapchain_context, frame_context)?;

        let cpu_frame_time_instant = Instant::now();

        let ui_snapshot = ui_context.build_ui_snapshot()?;

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
            Self::build_render_views_layout(&swapchain_context, &world_snapshot),
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

    fn build_render_views_layout(
        swapchain_context: &SwapchainContext,
        world_snapshot: &WorldSnapshot,
    ) -> RenderViewsLayout {
        let extent = swapchain_context.extent;
        let aspect_ratio = extent.width as f32 / extent.height as f32;
        let nearest_shadow_cascade_distance = 30.0;

        let main_projection = world_snapshot.camera_stamp.to_view_projection_matrix(aspect_ratio);
        let mut corners = Self::get_frustum_corners(main_projection);

        for i in 0..4 {
            let near = corners[i * 2];
            let far = corners[i * 2 + 1];
            let ray_dir = (far - near).normalize();

            corners[i * 2 + 1] = near + ray_dir * nearest_shadow_cascade_distance;
        }

        let mut frustum_center = Vec3::ZERO;
        for corner in &corners {
            frustum_center += *corner;
        }
        frustum_center /= 8.0;

        let light_direction = Vec3::new(0.6, -1.0, 0.3).normalize();
        let light_position = frustum_center - light_direction * (nearest_shadow_cascade_distance * 0.5);
        let light_view = Mat4::look_at_rh(light_position, frustum_center, Vec3::Z);

        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);

        for corner in corners {
            let light_space_projection = light_view * corner.extend(1.0);

            min = min.min(light_space_projection.xyz());
            max = max.max(light_space_projection.xyz());
        }

        let z_margin = 50.0;
        let shadow_far = -min.z;
        let shadow_near = -max.z - z_margin;
        let mut orthographic = Mat4::orthographic_rh(
            min.x, max.x,
            min.y, max.y,
            shadow_near, shadow_far,
        );
        orthographic.y_axis.y *= -1.0;

        let correction = Mat4::from_cols(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 0.5, 0.0),
            Vec4::new(0.0, 0.0, 0.5, 1.0),
        );

        let global_shadows_projection_matrix = correction * orthographic * light_view;

        RenderViewsLayout {
            main: RenderView {
                projection_matrix: main_projection,
            },
            global_shadow: RenderView {
                projection_matrix: global_shadows_projection_matrix,
            },
            shadows: Vec::new(),
        }
    }

    fn get_frustum_corners(projection: Mat4) -> Vec<Vec3> {
        let inverted_projection = projection.inverse();

        let mut corners = Vec::with_capacity(8);

        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                for z in [0.0, 1.0] {
                    let pt = inverted_projection * Vec4::new(x, y, z, 1.0);

                    corners.push(pt.xyz() / pt.w);
                }
            }
        }

        corners
    }

    pub fn destroy(
        mut self,
        device: &Device,
        index_managers: &IndexManagers,
        resource_factories: &ResourceFactories,
    ) -> Result<()> {
        self.render_stats_handler.destroy(&resource_factories.managed_buffer_factory)?;

        for render_pass in &self.render_passes {
            render_pass.destroy()?;
        }
        self.render_passes.clear();

        self.render_context.destroy(&device, &index_managers, &resource_factories)?;

        Ok(())
    }
}
