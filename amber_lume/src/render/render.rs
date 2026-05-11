use std::ptr::null_mut;
use crate::ids::{FrameIndex, SliceIndex};
use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::device::device_context::DeviceContext;
use crate::render::factories::image::image_view_description::ImageViewDescription;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::culling_indirect::cascade_culling_indirect_pass::CascadeCullingIndirectPass;
use crate::render::pass::culling_indirect::main_culling_indirect_pass::MainCullingIndirectPass;
use crate::render::pass::depth::depth_pass::DepthPass;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::main::main_pass::MainPass;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_layout::{RenderView, RenderViewsLayout};
use crate::render::pass::physics_debug::physics_debug_pass::PhysicsDebugPass;
use crate::render::pass::sdsm::cascade_compute_pass::CascadeComputePass;
use crate::render::pass::sdsm::sdsm_pass::SdsmPass;
use crate::render::pass::shadows::shadows_pass::ShadowsPass;
use crate::render::pass::skinning::skinning_pass::SkinningPass;
use crate::render::pass::ui::ui_render_pass::UiPass;
use crate::render::queue::queues::Queues;
use crate::render::render_context::RenderContext;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_graph::PassGraph;
use crate::render::render_graph::state::pass_graph_state::PassGraphState;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::image_size::ImageSize;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::renderer_statistics::{RenderStatistics, RenderStatisticsMeasurement};
use crate::render::resources::resource_context::ResourceContext;
use crate::render::statistics::interval::gpu_interval_measurement::GpuIntervalMeasurement;
use crate::render::statistics::pass_profiler::PassProfiler;
use crate::render::swapchain::swapchain_context::SwapchainContext;
use crate::resources::binding_layout::binding_layout::BindingLayout;
use crate::resources::binding_layout::descriptor_set_manager::GlobalDescriptorSetBindings;
use crate::resources::binding_layout::pipeline_layout_registry::PipelineLayoutType;
use crate::resources::resource_buffers::ResourceBuffers;
use crate::resources::sampler_registry::SamplerType;
use crate::resources::store::resource_store::ResourceStore;
use crate::settings::settings::EngineSettings;
use crate::snapshot_handler::render_snapshot::RenderSnapshot;
use crate::ui::ui_context::UiContext;
use crate::utils::matrix_wrappers::ViewProjectionMatrix;
use anyhow::{bail, Result};
use arc_swap::ArcSwap;
use ash::vk::{AccessFlags, Extent2D, Fence, Format, ImageAspectFlags, ImageLayout, ImageUsageFlags, PhysicalDevice, PipelineStageFlags, PresentInfoKHR, SubmitInfo};
use ash::{vk, Device, Instance};
use std::slice;
use std::sync::Arc;
use tracing::info;
use crate::limits::AmberLumeLimits;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::storage::render_persistent::RenderPersistent;
use crate::resources::resource_hub::ResourceHub;

pub struct Render {
    render_context: RenderContext,

    swapchain_image: VirtualImage,

    pass_graph: PassGraph,
    pass_profiler: PassProfiler,

    binding_layout: Arc<BindingLayout>,

    statistics: RenderStatisticsMeasurement,
    total_dispatch_measurement: GpuIntervalMeasurement,
}

impl Render {
    pub fn create(
        instance: &Instance,
        device_context: &DeviceContext,
        limits: &AmberLumeLimits,
        resource_factories: Arc<ResourceFactories>,
        settings: Arc<ArcSwap<EngineSettings>>,
        physical_device: PhysicalDevice,
        queues: &Queues,
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        resource_hub: Arc<ResourceHub>,
        resource_store: Arc<ResourceStore>,
        binding_layout: Arc<BindingLayout>,
    ) -> Result<Self> {
        let render_context = RenderContext::create(
            &instance,
            &device_context.device,
            &limits,
            physical_device,
            queues,
            &swapchain_context,
        )?;

        let pass_graph_state = PassGraphState::new();
        let mut pass_graph = PassGraph::new(pass_graph_state);

        let depth_image = pass_graph.create_image(
            "depth",
            ImageBlueprint {
                size: ImageSize::full_swapchain(),
                format: Format::D32_SFLOAT,
                usage: ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | ImageUsageFlags::SAMPLED,
                image_view_description: ImageViewDescription {
                    image_aspect_flags: ImageAspectFlags::DEPTH,
                    ..ImageViewDescription::default_2d_color()
                },
                descriptor: Some((GlobalDescriptorSetBindings::Texture, SamplerType::Depth)),
            },
        );
        let shadows_image = pass_graph.import_image(
            resource_hub.persistent_shadows.global_shadow_array.image,
            resource_hub.persistent_shadows.global_shadow_array.image_view,
            resource_hub.persistent_shadows
                .global_shadow_array
                .image_view_layers
                .clone(),
            Extent2D {
                width: resource_hub.persistent_shadows
                    .global_shadow_array
                    .image_description
                    .extent
                    .width,
                height: resource_hub.persistent_shadows
                    .global_shadow_array
                    .image_description
                    .extent
                    .height,
            },
            resource_hub.persistent_shadows.global_shadow_array.image_description.format,
            resource_hub.persistent_shadows
                .global_shadow_array
                .image_subresource_range,
            Some(
                resource_hub.persistent_shadows
                    .global_shadow_array_descriptor_id,
            ),
        );
        let swapchain_image = pass_graph.import_image_placeholder();

        let scene_buffer = pass_graph.import_buffer_placeholder();
        let entity_buffer = pass_graph.import_buffer_placeholder();
        let render_view_buffer = pass_graph.import_buffer_placeholder();
        let physics_debug_vertex_buffer = pass_graph.import_buffer_placeholder();
        let sdsm_result_buffer = pass_graph.import_buffer_placeholder();

        let shadow_cascades_buffer_view = resource_context.buffer_manager
            .shadow_cascades_buffer.as_view().slice_at(SliceIndex::ZERO);
        let shadow_cascades_buffer = pass_graph.import_buffer(
            shadow_cascades_buffer_view.handle(),
            shadow_cascades_buffer_view.offset(),
            shadow_cascades_buffer_view.size(),
            shadow_cascades_buffer_view.device_address(),
            null_mut(),
        );

        let main_culling_indirect_pass = MainCullingIndirectPass::create(
            &resource_context,
            &limits.resource_limits,
            limits.frames_in_flight,
            &resource_factories,
            &resource_store.compute_pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            scene_buffer,
            entity_buffer,
            render_view_buffer,
        )?;
        let skinning_pass = SkinningPass::create(
            &resource_store.compute_pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            resource_hub.bone_transform_handler.clone(),
        )?;
        let depth_pass = DepthPass::create(
            &resource_context,
            &render_context,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            depth_image,
            scene_buffer,
            entity_buffer,
        )?;
        let shadows_pass = ShadowsPass::create(
            &resource_context,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            &resource_hub.persistent_shadows,
            shadows_image,
            scene_buffer,
            entity_buffer,
            shadow_cascades_buffer,
        )?;
        let main_pass = MainPass::create(
            &resource_context,
            &swapchain_context,
            &render_context,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            swapchain_image,
            depth_image,
            shadows_image,
            scene_buffer,
            entity_buffer,
            shadow_cascades_buffer,
        )?;
        let physics_debug_pass = PhysicsDebugPass::create(
            &swapchain_context,
            &render_context,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            settings,
            swapchain_image,
            depth_image,
            physics_debug_vertex_buffer,
        )?;
        let ui_pass = UiPass::create(
            &swapchain_context,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            swapchain_image,
        )?;
        let sdsm_pass = SdsmPass::create(
            &resource_store.compute_pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            depth_image,
            sdsm_result_buffer,
        )?;
        let cascade_compute_pass = CascadeComputePass::create(
            &resource_store.compute_pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            limits.shadow_map_limits,
            scene_buffer,
            sdsm_result_buffer,
            render_view_buffer,
            shadow_cascades_buffer,
        )?;
        let cascade_culling_indirect_pass = CascadeCullingIndirectPass::create(
            &resource_context,
            &limits.resource_limits,
            limits.frames_in_flight,
            &resource_factories,
            &resource_store.compute_pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            scene_buffer,
            entity_buffer,
            render_view_buffer,
        )?;

        let mut pass_profiler = PassProfiler::new();
        pass_profiler.register(
            main_culling_indirect_pass.name(),
            &device_context,
            &resource_factories,
            limits.frames_in_flight,
        )?;
        pass_profiler.register(
            cascade_culling_indirect_pass.name(),
            &device_context,
            &resource_factories,
            limits.frames_in_flight,
        )?;
        pass_profiler.register(
            skinning_pass.name(),
            &device_context,
            &resource_factories,
            limits.frames_in_flight,
        )?;
        pass_profiler.register(
            depth_pass.name(),
            &device_context,
            &resource_factories,
            limits.frames_in_flight,
        )?;
        pass_profiler.register(
            shadows_pass.name(),
            &device_context,
            &resource_factories,
            limits.frames_in_flight,
        )?;
        pass_profiler.register(
            main_pass.name(),
            &device_context,
            &resource_factories,
            limits.frames_in_flight,
        )?;
        pass_profiler.register(
            physics_debug_pass.name(),
            &device_context,
            &resource_factories,
            limits.frames_in_flight,
        )?;
        pass_profiler.register(
            ui_pass.name(),
            &device_context,
            &resource_factories,
            limits.frames_in_flight,
        )?;
        pass_profiler.register(
            sdsm_pass.name(),
            &device_context,
            &resource_factories,
            limits.frames_in_flight,
        )?;
        pass_profiler.register(
            cascade_compute_pass.name(),
            &device_context,
            &resource_factories,
            limits.frames_in_flight,
        )?;

        pass_graph.add_pass(main_culling_indirect_pass);
        pass_graph.add_pass(skinning_pass);
        pass_graph.add_pass(depth_pass);
        pass_graph.add_pass(sdsm_pass);
        pass_graph.add_pass(cascade_compute_pass);
        pass_graph.add_pass(cascade_culling_indirect_pass);
        pass_graph.add_pass(shadows_pass);
        pass_graph.add_pass(main_pass);
        pass_graph.add_pass(physics_debug_pass);
        pass_graph.add_pass(ui_pass);

        pass_graph.build(
            swapchain_context.extent,
            &resource_factories,
            &resource_store.image_provider,
        )?;

        pass_profiler.set_order(pass_graph.order());

        let total_dispatch_measurement = GpuIntervalMeasurement::new(
            &device_context,
            "total_dispatch",
            &resource_factories.query_pool_factory,
            &resource_factories.buffer_factory,
            limits.frames_in_flight,
        )?;

        Ok(Self {
            render_context,

            swapchain_image,

            pass_graph,
            pass_profiler,

            binding_layout: binding_layout.clone(),

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
        limits: &AmberLumeLimits,
        resource_hub: &ResourceHub,
        buffer_manager: &BufferManager,
        resource_buffers: &ResourceBuffers,
        render_snapshot: Arc<RenderSnapshot>,
        persistent: &mut RenderPersistent,
    ) -> Result<()> {
        let frame_index = self.render_context.next_frame_index();
        let frame_context = self.render_context.get_frame(frame_index)?;

        unsafe {
            device_context
                .device
                .wait_for_fences(&[frame_context.fence], true, u64::MAX)?
        };

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

        let swapchain_image = swapchain_context.get_image(image_index)?;
        self.pass_graph.update_imported_image(
            self.swapchain_image,
            swapchain_image.image,
            swapchain_image.image_view,
            Vec::new(),
            swapchain_image.format,
            swapchain_image.extent,
            swapchain_image.image_subresource_range,
        );
        self.pass_graph.register_persistent_image(
            swapchain_image.image,
            ImageLayout::UNDEFINED,
            AccessFlags::empty(),
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        );

        persistent.cpu_to_gpu_allocator.begin_frame(frame_index);

        let render_views_layout =
            self.build_render_views_layout(&swapchain_context, &limits, &render_snapshot);
        let frame_data_context = FrameDataContext::create(
            frame_index,
            &device_context,
            &frame_context.command_recording,
            &limits,
            &render_views_layout,
            render_snapshot.clone(),
            ui_snapshot,
            &ui_context,
        );

        let render_pass_context = PassContext::create(
            &device_context,
            &swapchain_context,
            &self.render_context,
            &limits,
            &frame_context.command_recording,
            image_index,
            frame_index,
            &render_views_layout,
            &buffer_manager,
            &resource_buffers,
            &resource_hub.bone_transform_handler,
        )?;

        self.statistics.collect_record_commands.start();
        Self::collect_render_commands(
            &frame_data_context,
            &render_pass_context,
            &self.binding_layout,
            &self.total_dispatch_measurement,
            &mut self.pass_graph,
            &mut self.pass_profiler,
            &mut persistent.cpu_to_gpu_allocator,
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

        device_context
            .queues
            .submit_graphics(submit_info, frame_context.fence)?;

        let wait_semaphores = [present_semaphore];
        let swapchains = [swapchain_context.handle];
        let image_indices = [image_index];
        let present_info = PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let present_result = device_context
            .queues
            .present(&swapchain_context, present_info);

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
        binding_layout: &BindingLayout,
        total_dispatch_measurement: &GpuIntervalMeasurement,
        pass_graph: &mut PassGraph,
        pass_profiler: &mut PassProfiler,
        allocator: &mut HeapAllocator,
    ) -> Result<()> {
        pass_context.begin_command_recording()?;
        total_dispatch_measurement.record_start(
            pass_context.command_recording.command_buffer,
            pass_context.frame_index,
            0,
        );

        binding_layout.descriptor_set_manager.bind(
            pass_context.command_recording.command_buffer,
            binding_layout
                .pipeline_layout_registry
                .get(PipelineLayoutType::General),
        );

        pass_graph.run(
            &frame_data_context,
            &pass_context,
            pass_profiler,
            allocator,
        )?;

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
        limits: &AmberLumeLimits,
        render_snapshot: &RenderSnapshot,
    ) -> RenderViewsLayout {
        let extent = swapchain_context.extent;
        let aspect_ratio = extent.width as f32 / extent.height as f32;

        let camera_view = render_snapshot.camera.view();
        let camera_projection = render_snapshot.camera.projection(aspect_ratio);

        RenderViewsLayout {
            main: RenderView {
                view_projection: ViewProjectionMatrix::from_view_projection(
                    &camera_view,
                    &camera_projection,
                )
                .vulkan_corrected(),
            },
            cascade_count: limits.shadow_map_limits.cascade_count,
        }
    }

    pub fn statistics(&self, frame_index: FrameIndex, persistent: &RenderPersistent) -> RenderStatistics {
        RenderStatistics {
            total_time: self.statistics.total_time.collect(),
            collect_record_commands: self.statistics.collect_record_commands.collect(),

            total_dispatch: self.total_dispatch_measurement.collect(frame_index),

            cpu_to_gpu_allocator_statistics: persistent.cpu_to_gpu_allocator.statistics(),

            pass_profiles: self.pass_profiler.collect(frame_index),
        }
    }

    pub fn destroy(self, device: &Device, resource_factories: &ResourceFactories) -> Result<()> {
        self.total_dispatch_measurement
            .destroy(&resource_factories.buffer_factory)?;

        self.pass_profiler.destroy(&resource_factories)?;

        let pass_graph_state = self.pass_graph.destroy(&resource_factories)?;
        pass_graph_state.destroy(&resource_factories.managed_image_factory)?;

        self.render_context.destroy(&device)?;

        Ok(())
    }
}
