use std::array::from_fn;
use std::collections::HashMap;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicU64, Ordering};
use glam::{Mat4, Vec2, Vec3};
use crate::ids::SliceIndex;
use crate::render::device::device_context::DeviceContext;
use crate::render::factories::image::image_view_description::ImageViewDescription;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::bloom::bloom_downsample_pass::BloomDownsamplePass;
use crate::render::pass::bloom::bloom_upsample_pass::BloomUpsamplePass;
use crate::render::pass::culling_indirect::cascade_culling_indirect_pass::CascadeCullingIndirectPass;
use crate::render::pass::culling_indirect::main_culling_indirect_pass::MainCullingIndirectPass;
use crate::render::pass::debug_layer::debug_layer_pass::DebugLayerPass;
use crate::render::pass::depth::depth_prepass::DepthPrepass;
use crate::render::pass::environment::environment_pass::EnvironmentPass;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::fsr2::accumulate_pass::AccumulatePass;
use crate::render::pass::gtao::gtao_pass::GtaoPass;
use crate::render::pass::gtao::temporal_pass::GtaoTemporalPass;
use crate::render::pass::main::main_pass::MainPass;
use crate::render::pass::selection::selection_pass::SelectionPass;
use crate::render::readback::entity_id_pick_reader::EntityIdPickReader;
use crate::render::readback::readbacks::Readbacks;
use crate::render::readback::readback_pass::ReadbackPass;
use crate::utils::arc_utils::ArcUnwrapOrErr;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_layout::{RenderView, RenderViewsLayout};
use crate::render::pass::physics_debug::physics_debug_pass::PhysicsDebugPass;
use crate::render::pass::sdsm::cascade_compute_pass::CascadeComputePass;
use crate::render::pass::sdsm::sdsm_pass::SdsmPass;
use crate::render::pass::shadows::shadows_pass::ShadowsPass;
use crate::render::pass::skinning::skinning_pass::SkinningPass;
use crate::render::pass::tonemap::tonemap_pass::TonemapPass;
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
use crate::resources::binding_layout::pipeline_layout_registry::PipelineLayoutType;
use crate::resources::resource_buffers::ResourceBuffers;
use crate::resources::store::resource_store::ResourceStore;
use crate::settings::settings::EngineSettings;
use crate::snapshot_handler::render_snapshot::{RenderEntityId, RenderSnapshot};
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
use crate::render::buffer::typed::draw_data_buffer::DrawDataGPU;
use crate::render::buffer::typed::indirect_buffer::IndirectGPU;
use crate::render::frame_data::bone_transform::BoneTransformGPU;
use crate::render::render_graph::virtual_buffer::buffer_blueprint::BufferBlueprint;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::state::render_state::RenderState;
use crate::render::swapchain::surface_format::HDR_FORMAT;
use crate::render::target::render_target::RenderTarget;
use crate::resources::skinning::bone_transform_handler::BoneTransformHandler;

const JITTER_PHASE: u64 = 16;

pub struct Render {
    pub target: Arc<dyn RenderTarget>,

    render_context: RenderContext,

    render_extent: Extent2D,

    target_image: VirtualImage,

    pass_graph: PassGraph,

    render_state: RenderState,
    binding_layout: Arc<BindingLayout>,

    profiler: Arc<FrameProfiler>,

    readbacks: Arc<Readbacks>,
    pick_reader: Arc<EntityIdPickReader>,

    previous_view_projection: Option<ViewProjectionMatrix>,
    previous_transforms: HashMap<RenderEntityId, Mat4>,

    frame_counter: Arc<AtomicU64>,

    settings: Arc<ArcSwap<EngineSettings>>,
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
        frame_counter: Arc<AtomicU64>,
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
        let scene_color_format = Format::R16G16B16A16_SFLOAT;
        let target_extent = target.extent();
        let render_scale = settings.load().render.render_scale.value;
        let render_extent = Self::scaled_render_extent(target_extent, render_scale);

        let pass_graph_state = render_state.pass_graph_state
            .take()
            .expect("pass graph state");
        let mut pass_graph = PassGraph::new(pass_graph_state);

        let depth_image = pass_graph.create_image(
            "depth",
            ImageBlueprint {
                size: ImageSize::render_full(),
                array_layers: 1,
                format: Format::D32_SFLOAT,
                usage: ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                image_view_description: ImageViewDescription {
                    image_aspect_flags: ImageAspectFlags::DEPTH,
                    ..ImageViewDescription::default_2d_color()
                },
                sampled: true,
            },
        );
        let normal_image = pass_graph.create_image(
            "normal",
            ImageBlueprint {
                size: ImageSize::render_full(),
                array_layers: 1,
                format: Format::R16G16B16A16_SFLOAT,
                usage: ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                image_view_description: ImageViewDescription::default_2d_color(),
                sampled: true,
            },
        );
        let velocity_image = pass_graph.create_image(
            "velocity",
            ImageBlueprint {
                size: ImageSize::render_full(),
                array_layers: 1,
                format: Format::R16G16_SFLOAT,
                usage: ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                image_view_description: ImageViewDescription::default_2d_color(),
                sampled: true,
            },
        );
        let gtao_image = pass_graph.create_image(
            "gtao",
            ImageBlueprint {
                size: ImageSize::Render { pow: 1 },
                array_layers: 1,
                format: Format::R16_SFLOAT,
                usage: ImageUsageFlags::STORAGE | ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                image_view_description: ImageViewDescription::default_2d_color(),
                sampled: true,
            },
        );
        let gtao_resolved_image = pass_graph.create_image(
            "gtao_resolved",
            ImageBlueprint {
                size: ImageSize::Render { pow: 1 },
                array_layers: 1,
                format: Format::R16_SFLOAT,
                usage: ImageUsageFlags::STORAGE | ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                image_view_description: ImageViewDescription::default_2d_color(),
                sampled: true,
            },
        );
        let gtao_history_images: [VirtualImage; 2] = from_fn(|index| {
            pass_graph.create_image(
                if index == 0 { "gtao_history_a" } else { "gtao_history_b" },
                ImageBlueprint {
                    size: ImageSize::Render { pow: 1 },
                    array_layers: 1,
                    format: Format::R16_SFLOAT,
                    usage: ImageUsageFlags::STORAGE | ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                    image_view_description: ImageViewDescription::default_2d_color(),
                    sampled: true,
                },
            )
        });
        let entity_id_image = pass_graph.create_image(
            "entity_id",
            ImageBlueprint {
                size: ImageSize::render_full(),
                array_layers: 1,
                format: Format::R32_UINT,
                usage: ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::TRANSFER_SRC | ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED,
                image_view_description: ImageViewDescription::default_2d_color(),
                sampled: true,
            },
        );
        let scene_color_image = pass_graph.create_image(
            "scene_color",
            ImageBlueprint {
                size: ImageSize::render_full(),
                array_layers: 1,
                format: scene_color_format,
                usage: ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                image_view_description: ImageViewDescription::default_2d_color(),
                sampled: true,
            },
        );
        let history_images: [VirtualImage; 2] = from_fn(|index| {
            pass_graph.create_image(
                if index == 0 { "history_a" } else { "history_b" },
                ImageBlueprint {
                    size: ImageSize::Target { pow: 0 },
                    array_layers: 1,
                    format: scene_color_format,
                    usage: ImageUsageFlags::STORAGE | ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                    image_view_description: ImageViewDescription::default_2d_color(),
                    sampled: true,
                },
            )
        });

        const BLOOM_MIPS: usize = 5;
        let bloom_labels: [&'static str; BLOOM_MIPS] =
            ["bloom_1", "bloom_2", "bloom_3", "bloom_4", "bloom_5"];
        let bloom_mips: [VirtualImage; BLOOM_MIPS] = from_fn(|index| {
            pass_graph.create_image(
                bloom_labels[index],
                ImageBlueprint {
                    size: ImageSize::Render { pow: (index + 1) as u32 },
                    array_layers: 1,
                    format: scene_color_format,
                    usage: ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                    image_view_description: ImageViewDescription::default_2d_color(),
                    sampled: true,
                },
            )
        });
        let shadow_physical = render_state.image_scope.get_physical_image(render_state.shadow_image);
        let shadow_descriptor = render_state.bindless.shadow_arrays
            .acquire_image(shadow_physical.image_view);
        let shadows_image = pass_graph.import_image(
            "global_shadow_array",
            shadow_physical.image,
            shadow_physical.image_view,
            shadow_physical.extent,
            shadow_physical.format,
            shadow_physical.subresource_range,
            shadow_descriptor,
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

        let indirect_blueprint = BufferBlueprint::new(
            (size_of::<IndirectGPU>() * limits.resource_limits.max_draw_calls as usize) as DeviceSize,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::INDIRECT_BUFFER,
        );
        let indirect_main = pass_graph.create_buffer("indirect_main", indirect_blueprint);
        let indirect_shadow = pass_graph.create_buffer("indirect_shadow", indirect_blueprint);

        let draw_data_blueprint = BufferBlueprint::new(
            (size_of::<DrawDataGPU>() * limits.resource_limits.max_draw_calls as usize) as DeviceSize,
            BufferUsageFlags::STORAGE_BUFFER,
        );
        let draw_data_main = pass_graph.create_buffer("draw_data_main", draw_data_blueprint);
        let draw_data_shadow = pass_graph.create_buffer("draw_data_shadow", draw_data_blueprint);

        let bone_transform = pass_graph.create_buffer(
            "bone_transform",
            BufferBlueprint::new(
                (size_of::<BoneTransformGPU>() * limits.resource_limits.max_bone_transforms as usize) as DeviceSize,
                BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST,
            ),
        );

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
            indirect_main,
            indirect_shadow,
            draw_data_main,
            draw_data_shadow,
        )?;
        let skinning_pass = SkinningPass::create(
            &resource_store.compute_pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            bone_transform_handler.clone(),
            bone_transform,
        )?;
        let shadows_pass = ShadowsPass::create(
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            limits.shadow_map_limits.cascade_count,
            limits.shadow_map_limits.format.vulkan(),
            shadows_image,
            entity_buffer,
            shadow_cascades_buffer,
            draw_count_shadow,
            indirect_shadow,
            draw_data_shadow,
            bone_transform,
        )?;
        let environment_pass = EnvironmentPass::create(
            scene_color_format,
            &render_context,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            scene_color_image,
            depth_image,
        )?;
        let depth_prepass = DepthPrepass::create(
            &render_context,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            depth_image,
            normal_image,
            Format::R16G16B16A16_SFLOAT,
            velocity_image,
            Format::R16G16_SFLOAT,
            scene_buffer,
            entity_buffer,
            draw_count_main,
            indirect_main,
            draw_data_main,
            bone_transform,
        )?;
        let main_pass = MainPass::create(
            scene_color_format,
            &render_context,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            scene_color_image,
            entity_id_image,
            depth_image,
            shadows_image,
            gtao_resolved_image,
            scene_buffer,
            entity_buffer,
            shadow_cascades_buffer,
            draw_count_main,
            indirect_main,
            draw_data_main,
            bone_transform,
            settings.clone(),
        )?;
        let mut bloom_downsample_passes = Vec::with_capacity(BLOOM_MIPS);
        bloom_downsample_passes.push(BloomDownsamplePass::create(
            scene_color_format,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            scene_color_image,
            bloom_mips[0],
            true,
            settings.clone(),
        )?);
        for index in 1..BLOOM_MIPS {
            bloom_downsample_passes.push(BloomDownsamplePass::create(
                scene_color_format,
                &resource_store.pipeline_provider,
                &binding_layout.pipeline_layout_registry,
                bloom_mips[index - 1],
                bloom_mips[index],
                false,
                settings.clone(),
            )?);
        }

        let mut bloom_upsample_passes = Vec::with_capacity(BLOOM_MIPS - 1);
        for index in (0..BLOOM_MIPS - 1).rev() {
            bloom_upsample_passes.push(BloomUpsamplePass::create(
                scene_color_format,
                &resource_store.pipeline_provider,
                &binding_layout.pipeline_layout_registry,
                bloom_mips[index + 1],
                bloom_mips[index],
                settings.clone(),
            )?);
        }

        let accumulate_pass = AccumulatePass::create(
            &resource_store.compute_pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            scene_color_image,
            velocity_image,
            history_images[0],
            history_images[1],
            settings.clone(),
        )?;
        let tonemap_pass = TonemapPass::create(
            color_format,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            scene_color_image,
            history_images[0],
            history_images[1],
            bloom_mips[0],
            target_image,
            settings.clone(),
            color_format == HDR_FORMAT,
        )?;
        let debug_layer_pass = DebugLayerPass::create(
            color_format,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            velocity_image,
            normal_image,
            gtao_image,
            target_image,
            settings.clone(),
        )?;
        let physics_debug_pass = PhysicsDebugPass::create(
            color_format,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            settings.clone(),
            target_image,
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
        let gtao_pass = GtaoPass::create(
            &resource_store.compute_pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            depth_image,
            normal_image,
            gtao_image,
            scene_buffer,
            settings.clone(),
        )?;
        let gtao_temporal_pass = GtaoTemporalPass::create(
            &resource_store.compute_pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            gtao_image,
            velocity_image,
            gtao_history_images[0],
            gtao_history_images[1],
            gtao_resolved_image,
            settings.clone(),
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
            &limits.resource_limits,
            limits.frames_in_flight,
            &resource_factories,
            &resource_store.compute_pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            scene_buffer,
            entity_buffer,
            render_view_buffer,
            draw_count_shadow,
            indirect_shadow,
            draw_data_shadow,
        )?;

        let pick_reader = Arc::new(EntityIdPickReader::create(entity_id_image));

        let selection_pass = SelectionPass::create(
            color_format,
            &resource_store.pipeline_provider,
            &binding_layout.pipeline_layout_registry,
            target_image,
            entity_id_image,
            [1.0, 0.5, 0.0, 0.15],
            settings.clone(),
            pick_reader.clone(),
        )?;

        let readbacks = Arc::new(Readbacks::new(
            &resource_factories.buffer_factory,
            vec![pick_reader.clone()],
            limits.frames_in_flight,
        )?);
        let readback_pass = ReadbackPass::new(readbacks.clone());

        pass_graph.add_pass(main_culling_indirect_pass, &profiler);
        pass_graph.add_pass(skinning_pass, &profiler);
        pass_graph.add_pass(sdsm_pass, &profiler);
        pass_graph.add_pass(cascade_compute_pass, &profiler);
        pass_graph.add_pass(cascade_culling_indirect_pass, &profiler);
        pass_graph.add_pass(shadows_pass, &profiler);
        pass_graph.add_pass(depth_prepass, &profiler);
        pass_graph.add_pass(gtao_pass, &profiler);
        pass_graph.add_pass(gtao_temporal_pass, &profiler);
        pass_graph.add_pass(environment_pass, &profiler);
        pass_graph.add_pass(main_pass, &profiler);
        pass_graph.add_pass(accumulate_pass, &profiler);
        for pass in bloom_downsample_passes {
            pass_graph.add_pass(pass, &profiler);
        }
        for pass in bloom_upsample_passes {
            pass_graph.add_pass(pass, &profiler);
        }
        pass_graph.add_pass(tonemap_pass, &profiler);
        pass_graph.add_pass(debug_layer_pass, &profiler);
        pass_graph.add_pass(selection_pass, &profiler);
        pass_graph.add_pass(physics_debug_pass, &profiler);
        pass_graph.add_pass(ui_pass, &profiler);
        pass_graph.add_pass(readback_pass, &profiler);

        pass_graph.build(
            target_extent,
            render_extent,
            &resource_factories,
            &render_state.bindless.graph_textures,
            &render_state.bindless.storage_images,
            limits.frames_in_flight,
        )?;

        Ok(Self {
            target,

            render_context,

            render_extent,

            target_image,

            pass_graph,

            render_state,
            binding_layout,

            profiler,

            readbacks,
            pick_reader,

            previous_view_projection: None,
            previous_transforms: HashMap::new(),

            frame_counter,

            settings,
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

        self.render_state.bindless.update();

        let Some(image_index) = self.target.acquire_next_image(frame_context.acquire_semaphore)? else {
            return Ok(());
        };

        self.profiler.begin_frame(frame_index);

        self.readbacks.sync(frame_index);

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
        let mut render_views_layout = self.build_render_views_layout(target_extent, &limits, &render_snapshot);

        let current_main_view_projection = render_views_layout.main.view_projection;
        render_views_layout.main.previous_view_projection =
            self.previous_view_projection.unwrap_or(current_main_view_projection);

        let previous_transforms: Vec<Mat4> = render_snapshot
            .entities
            .iter()
            .map(|entity| {
                self.previous_transforms
                    .get(&entity.id)
                    .copied()
                    .unwrap_or(entity.transform_matrix)
            })
            .collect();

        let frame_data_context = FrameDataContext::create(
            frame_index,
            &device_context,
            &frame_context.command_recording,
            &limits,
            &render_views_layout,
            render_snapshot.clone(),
            previous_transforms,
            ui_snapshot,
            &ui_context,
        );

        let frame_number = self.frame_counter.load(Ordering::Relaxed);
        let history_write_index = (frame_number & 1) as u32;
        let history_valid = frame_number != 0;

        let render_pass_context = PassContext::create(
            &device_context,
            &self.render_context,
            &limits,
            &frame_context.command_recording,
            target_image,
            frame_index,
            frame_number as u32,
            history_write_index,
            history_valid,
            &render_views_layout,
            &resource_buffers,
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

        self.previous_view_projection = Some(current_main_view_projection);
        self.previous_transforms = render_snapshot
            .entities
            .iter()
            .map(|entity| (entity.id, entity.transform_matrix))
            .collect();

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

        let tan_half_fov_x = 1.0 / camera_projection.value.x_axis.x;
        let tan_half_fov_y = 1.0 / camera_projection.value.y_axis.y;

        let view_projection = ViewProjectionMatrix::from_view_projection(
            &camera_view,
            &camera_projection,
        )
        .vulkan_corrected();

        let render_width = self.render_extent.width.max(1) as f32;
        let render_height = self.render_extent.height.max(1) as f32;

        let jittered_view_projection = if self.settings.load().render.fsr_enabled.value {
            let jitter_index = (self.frame_counter.load(Ordering::Relaxed) % JITTER_PHASE) as u32 + 1;
            let jitter_ndc_x = (Self::halton(jitter_index, 2) - 0.5) * 2.0 / render_width;
            let jitter_ndc_y = (Self::halton(jitter_index, 3) - 0.5) * 2.0 / render_height;

            ViewProjectionMatrix {
                value: Mat4::from_translation(Vec3::new(jitter_ndc_x, jitter_ndc_y, 0.0)) * view_projection.value,
            }
        } else {
            view_projection
        };

        let mip_bias = (render_width / extent.width.max(1) as f32).log2();

        RenderViewsLayout {
            main: RenderView {
                view_projection,
                view: camera_view,

                ndc_to_view_mul: Vec2::new(2.0 * tan_half_fov_x, -2.0 * tan_half_fov_y),
                ndc_to_view_add: Vec2::new(-tan_half_fov_x, tan_half_fov_y),

                previous_view_projection: view_projection,
                jittered_view_projection,
                mip_bias,
            },
            cascade_count: limits.shadow_map_limits.cascade_count,
        }
    }

    fn halton(index: u32, base: u32) -> f32 {
        let mut result = 0.0;
        let mut fraction = 1.0;
        let mut i = index;

        while i > 0 {
            fraction /= base as f32;
            result += fraction * (i % base) as f32;
            i /= base;
        }

        result
    }

    pub fn statistics(&self) -> RenderStatistics {
        RenderStatistics {
            cpu_to_gpu_allocator_statistics: self.render_state.cpu_to_gpu_allocator.statistics(),
            hdr_supported: self.target.hdr_supported(),
        }
    }

    pub fn picked_entity(&self) -> Option<u32> {
        self.pick_reader.value()
    }

    fn scaled_render_extent(target_extent: Extent2D, render_scale: f32) -> Extent2D {
        let scale = render_scale.clamp(0.1, 1.0);

        Extent2D {
            width: ((target_extent.width as f32 * scale).round() as u32).max(1),
            height: ((target_extent.height as f32 * scale).round() as u32).max(1),
        }
    }

    pub fn render_resolution_out_of_date(&self, render_scale: f32) -> bool {
        Self::scaled_render_extent(self.target.extent(), render_scale) != self.render_extent
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
        let frame_counter = self.frame_counter.clone();
        let hdr = settings.load().render.hdr.value && target.hdr_supported();
        target.invalidate(vulkan_context, device_context, hdr)?;

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
            frame_counter,
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
            readbacks,
            ..
        } = self;

        render_state.pass_graph_state = Some(pass_graph.destroy(&resource_factories)?);

        readbacks.try_unwrap()?.destroy(&resource_factories.buffer_factory)?;

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
