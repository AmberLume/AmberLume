use std::ptr::null_mut;
use crate::ids::SliceIndex;
use crate::render::device::device_context::DeviceContext;
use crate::render::factories::image::image_view_description::ImageViewDescription;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::culling_indirect::cascade_culling_indirect_pass::CascadeCullingIndirectPass;
use crate::render::pass::culling_indirect::main_culling_indirect_pass::MainCullingIndirectPass;
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
use crate::render::render_graph::pass_graph::PassGraph;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::image_size::ImageSize;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::{profile_cpu_meta, profile_cpu_zone};
use crate::profile_gpu_zone;
use crate::profiler::frame_profiler::FrameProfiler;
use crate::render::renderer_statistics::RenderStatistics;
use crate::render::resources::resource_context::ResourceContext;
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
use anyhow::Result;
use arc_swap::ArcSwap;
use ash::vk::{AccessFlags, BufferUsageFlags, DeviceSize, Extent2D, Format, ImageAspectFlags, ImageLayout, ImageUsageFlags, PhysicalDevice, PipelineStageFlags, SubmitInfo};
use ash::{Device, Instance};
use std::slice;
use std::sync::Arc;
use tracing::info;
use crate::limits::AmberLumeLimits;
use crate::render::device::vulkan_context::VulkanContext;
use crate::render::render_graph::virtual_buffer::buffer_blueprint::BufferBlueprint;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::state::render_state::RenderState;
use crate::render::target::render_target::RenderTarget;
use crate::resources::skinning::bone_transform_handler::BoneTransformHandler;

pub struct Render {
    pub target: Arc<dyn RenderTarget>,

    render_context: RenderContext,

    target_image: VirtualImage,

    pass_graph: PassGraph,

    render_state: RenderState,
    binding_layout: Arc<BindingLayout>,
    bone_transform_handler: Arc<BoneTransformHandler>,

    profiler: Arc<FrameProfiler>,
}

impl Render {
    pub fn create(
        instance: &Instance,
        device_context: &DeviceContext,
        limits: &AmberLumeLimits,
        target: Arc<dyn RenderTarget>,
        resource_factories: Arc<ResourceFactories>,
        settings: Arc<ArcSwap<EngineSettings>>,
        physical_device: PhysicalDevice,
        queues: &Queues,
        resource_context: &ResourceContext,
        resource_store: Arc<ResourceStore>,
        binding_layout: Arc<BindingLayout>,
        bone_transform_handler: Arc<BoneTransformHandler>,
        profiler: Arc<FrameProfiler>,
        mut render_state: RenderState,
    ) -> Result<Self> {
        let render_context = RenderContext::create(
            &instance,
            &device_context.device,
            &limits,
            physical_device,
            queues,
        )?;

        let color_format = target.format();
        let target_extent = target.extent();

        let pass_graph_state = render_state.pass_graph_state
            .take()
            .expect("pass graph state");
        let mut pass_graph = PassGraph::new(pass_graph_state);

        let depth_image = pass_graph.create_image(
            "depth",
            ImageBlueprint {
                size: ImageSize::full(),
                format: Format::D32_SFLOAT,
                usage: ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                image_view_description: ImageViewDescription {
                    image_aspect_flags: ImageAspectFlags::DEPTH,
                    ..ImageViewDescription::default_2d_color()
                },
                descriptor: Some((GlobalDescriptorSetBindings::Texture, SamplerType::Depth)),
            },
        );
        let shadows_image = pass_graph.import_image(
            "global_shadow_array",
            render_state.persistent_shadows.global_shadow_array.image,
            render_state.persistent_shadows.global_shadow_array.image_view,
            Extent2D {
                width: render_state.persistent_shadows
                    .global_shadow_array
                    .image_description
                    .extent
                    .width,
                height: render_state.persistent_shadows
                    .global_shadow_array
                    .image_description
                    .extent
                    .height,
            },
            render_state.persistent_shadows.global_shadow_array.image_description.format,
            render_state.persistent_shadows
                .global_shadow_array
                .image_subresource_range,
            Some(
                render_state.persistent_shadows
                    .global_shadow_array_descriptor_id,
            ),
        );
        let target_image = pass_graph.import_image_placeholder("render_target");

        let scene_buffer = pass_graph.import_buffer_placeholder("scene");
        let entity_buffer = pass_graph.import_buffer_placeholder("entity");
        let render_view_buffer = pass_graph.import_buffer_placeholder("render_view");
        let physics_debug_vertex_buffer = pass_graph.import_buffer_placeholder("physics_debug_vertex");
        let sdsm_result_buffer = pass_graph.import_buffer_placeholder("sdsm_result");

        let draw_count_blueprint = BufferBlueprint::new(
            size_of::<u32>() as DeviceSize,
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::TRANSFER_DST
                | BufferUsageFlags::INDIRECT_BUFFER,
        );
        let draw_count_main = pass_graph.create_buffer("draw_count_main", draw_count_blueprint);
        let draw_count_shadow = pass_graph.create_buffer("draw_count_shadow", draw_count_blueprint);

        let shadow_cascades_buffer_view = resource_context.buffer_manager
            .shadow_cascades_buffer.as_view().slice_at(SliceIndex::ZERO);
        let shadow_cascades_buffer = pass_graph.import_buffer(
            "shadow_cascades",
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
            draw_count_main,
            draw_count_shadow,
        )?;
        let skinning_pass = SkinningPass::create(
            &resource_store.compute_pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            bone_transform_handler.clone(),
        )?;
        let shadows_pass = ShadowsPass::create(
            &resource_context,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            &render_state.persistent_shadows,
            shadows_image,
            entity_buffer,
            shadow_cascades_buffer,
            draw_count_shadow,
        )?;
        let main_pass = MainPass::create(
            &resource_context,
            color_format,
            &render_context,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            target_image,
            depth_image,
            shadows_image,
            scene_buffer,
            entity_buffer,
            shadow_cascades_buffer,
            draw_count_main,
        )?;
        let physics_debug_pass = PhysicsDebugPass::create(
            color_format,
            &render_context,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            settings,
            target_image,
            depth_image,
            physics_debug_vertex_buffer,
        )?;
        let ui_pass = UiPass::create(
            color_format,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            target_image,
        )?;
        let sdsm_pass = SdsmPass::create(
            &resource_store.compute_pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            depth_image,
            sdsm_result_buffer,
            limits.shadow_map_limits.z_far_sample_stride,
        )?;
        let cascade_compute_pass = CascadeComputePass::create(
            &resource_store.compute_pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            limits.shadow_map_limits,
            &resource_factories,
            limits.frames_in_flight,
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
            draw_count_shadow,
        )?;

        pass_graph.add_pass(main_culling_indirect_pass, &profiler);
        pass_graph.add_pass(skinning_pass, &profiler);
        pass_graph.add_pass(sdsm_pass, &profiler);
        pass_graph.add_pass(cascade_compute_pass, &profiler);
        pass_graph.add_pass(cascade_culling_indirect_pass, &profiler);
        pass_graph.add_pass(shadows_pass, &profiler);
        pass_graph.add_pass(main_pass, &profiler);
        pass_graph.add_pass(physics_debug_pass, &profiler);
        pass_graph.add_pass(ui_pass, &profiler);

        pass_graph.build(
            target_extent,
            &resource_factories,
            &resource_store.image_provider,
            limits.frames_in_flight,
        )?;

        Ok(Self {
            target,

            render_context,

            target_image,

            pass_graph,

            render_state,
            binding_layout,
            bone_transform_handler,

            profiler,
        })
    }

    pub fn render_frame(
        &mut self,
        device_context: &DeviceContext,
        ui_context: &mut UiContext,
        limits: &AmberLumeLimits,
        resource_buffers: &ResourceBuffers,
        render_snapshot: Arc<RenderSnapshot>,
    ) -> Result<()> {
        let frame_index = self.render_context.next_frame_index();
        let frame_context = self.render_context.get_frame(frame_index)?;

        unsafe {
            device_context
                .device
                .wait_for_fences(&[frame_context.fence], true, u64::MAX)?
        };

        let Some(image_index) = self.target.acquire_next_image(frame_context.acquire_semaphore)? else {
            return Ok(());
        };

        self.profiler.begin_frame(frame_index);

        let skinned_entities = render_snapshot
            .entities
            .iter()
            .filter(|entity| entity.animation.is_some())
            .count() as u32;
        profile_cpu_meta!(&self.profiler, "world.entities", render_snapshot.entities.len() as u32);
        profile_cpu_meta!(&self.profiler, "world.skinned_entities", skinned_entities);
        profile_cpu_meta!(&self.profiler, "world.physics_debug_lines", render_snapshot.physics_debug_lines.len() as u32);

        let ui_snapshot = profile_cpu_zone!(&self.profiler, "ui.build_snapshot", {
            ui_context.build_ui_snapshot()?
        });

        let target_image = self.target.get_image(image_index)?;
        self.pass_graph.rebind_image(
            self.target_image,
            target_image.image,
            target_image.image_view,
            target_image.extent,
            target_image.format,
            target_image.image_subresource_range,
            None,
        );
        self.pass_graph.register_persistent_image(
            target_image.image,
            ImageLayout::UNDEFINED,
            AccessFlags::empty(),
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        );

        self.render_state.cpu_to_gpu_allocator.begin_frame(frame_index);
        self.pass_graph.begin_transient_buffers_frame(frame_index);

        let target_extent = self.target.extent();
        let render_views_layout = self.build_render_views_layout(target_extent, &limits, &render_snapshot);
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
            &self.render_context,
            &limits,
            &frame_context.command_recording,
            target_image,
            frame_index,
            &render_views_layout,
            &resource_buffers,
            &self.bone_transform_handler,
        )?;

        profile_cpu_zone!(&self.profiler, "render.collect_commands", {
            Self::collect_render_commands(
                &frame_data_context,
                &render_pass_context,
                &self.binding_layout,
                &self.profiler,
                &mut self.pass_graph,
                &mut self.render_state.cpu_to_gpu_allocator,
            )?;
        });

        let cpu_to_gpu = self.render_state.cpu_to_gpu_allocator.statistics();
        profile_cpu_meta!(&self.profiler, "render.cpu_to_gpu.used", cpu_to_gpu.used);
        profile_cpu_meta!(&self.profiler, "render.cpu_to_gpu.capacity", cpu_to_gpu.capacity);

        let present_semaphore = self.target.get_present_semaphore(image_index)?;

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

        self.target.present(&device_context.queues, image_index, present_semaphore)?;

        self.profiler.end_frame();

        Ok(())
    }

    fn collect_render_commands(
        frame_data_context: &FrameDataContext,
        pass_context: &PassContext,
        binding_layout: &BindingLayout,
        profiler: &FrameProfiler,
        pass_graph: &mut PassGraph,
        allocator: &mut HeapAllocator,
    ) -> Result<()> {
        let command_buffer = pass_context.command_recording.command_buffer;

        pass_context.begin_command_recording()?;

        binding_layout.descriptor_set_manager.bind(
            command_buffer,
            binding_layout
                .pipeline_layout_registry
                .get(PipelineLayoutType::General),
        );

        profile_gpu_zone!(profiler, command_buffer, "render.total_dispatch", {
            pass_graph.run(
                &frame_data_context,
                &pass_context,
                profiler,
                allocator,
            )?;
        });

        profiler.extract_queries(command_buffer, pass_context.frame_index);

        pass_context.end_command_recording()?;

        Ok(())
    }

    fn build_render_views_layout(
        &self,
        extent: Extent2D,
        limits: &AmberLumeLimits,
        render_snapshot: &RenderSnapshot,
    ) -> RenderViewsLayout {
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

    pub fn statistics(&self) -> RenderStatistics {
        RenderStatistics {
            cpu_to_gpu_allocator_statistics: self.render_state.cpu_to_gpu_allocator.statistics(),
        }
    }

    pub fn invalidate(
        self,
        instance: &Instance,
        vulkan_context: &VulkanContext,
        device_context: &DeviceContext,
        limits: &AmberLumeLimits,
        resource_factories: Arc<ResourceFactories>,
        settings: Arc<ArcSwap<EngineSettings>>,
        physical_device: PhysicalDevice,
        resource_context: &ResourceContext,
        binding_layout: Arc<BindingLayout>,
        bone_transform_handler: Arc<BoneTransformHandler>,
        resource_store: Arc<ResourceStore>,
    ) -> Result<Self> {
        let target = self.target.clone();
        let profiler = self.profiler.clone();
        target.invalidate(vulkan_context, device_context)?;

        let render_state = self.destroy_inner(&device_context.device, &resource_factories)?;

        let render = Self::create(
            instance,
            device_context,
            limits,
            target,
            resource_factories.clone(),
            settings,
            physical_device,
            &device_context.queues,
            resource_context,
            resource_store,
            binding_layout,
            bone_transform_handler,
            profiler.clone(),
            render_state,
        )?;

        profiler.flush_pending_provider_destroy(&resource_factories)?;

        Ok(render)
    }

    fn destroy_inner(self, device: &Device, resource_factories: &ResourceFactories) -> Result<RenderState> {
        let Self {
            render_context,
            pass_graph,
            mut render_state,
            ..
        } = self;

        render_state.pass_graph_state = Some(pass_graph.destroy(&resource_factories)?);
        render_context.destroy(&device)?;

        info!("Render destroyed");

        Ok(render_state)
    }
    
    pub fn destroy(
        self,
        vulkan_context: &VulkanContext,
        device_context: &DeviceContext,
        resource_factories: &ResourceFactories,
    ) -> Result<RenderState> {
        let target = self.target.clone();
        let render_state = self.destroy_inner(&device_context.device, resource_factories)?;
        target.destroy_resources(vulkan_context, device_context)?;

        Ok(render_state)
    }
}
