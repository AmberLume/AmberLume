use gpu_data::MaterialGPU;
use crate::limits::RenderLimits;
use gpu::profile_gpu_zone;
use gpu::FrameProfiler;
use crate::render::frame_data::draw_data_buffer::DrawDataGPU;
use crate::render::frame_data::indirect_buffer::IndirectGPU;
use gpu::DeviceContext;
use gpu::VulkanContext;
use gpu::ImageViewDescription;
use gpu::ResourceFactories;
use crate::render::frame_data::bone_transform::BoneTransformGPU;
use crate::render::pass::ao::Ao;
use crate::render::pass::blas_build::blas_build_pass::BLASBuildPass;
use crate::render::pass::bloom::bloom_downsample_pass::BloomDownsamplePass;
use crate::render::pass::bloom::bloom_upsample_pass::BloomUpsamplePass;
use crate::render::pass::brdf_lut::brdf_lut_pass::BrdfLutPass;
use crate::render::pass::culling_indirect::cull_request::CullRequest;
use crate::render::pass::draw_bucket::DrawBucket;
use crate::render::pass::draw_pool::DrawPool;
use crate::render::pass::culling_indirect::culling_indirect_pass::CullingIndirectPass;
use crate::render::pass::culling_indirect::cull_request_statistics::MAIN_CULLING_META_NAME;
use crate::render::pass::frame_staging::frame_staging_pass::FrameStagingPass;
use crate::render::pass::draw_sort::draw_sort_pass::DrawSortPass;
use crate::render::pass::transparent::transparent_pass::TransparentPass;
use crate::render::pass::transparent_entity_id::transparent_entity_id_pass::TransparentEntityIdPass;

use crate::render::pass::debug_layer::debug_layer_pass::DebugLayerPass;
use crate::render::pass::depth::depth_prepass::DepthPrepass;
use crate::render::pass::environment::environment_pass::EnvironmentPass;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::fsr2::accumulate_pass::AccumulatePass;
use crate::render::pass::hiz::hiz_pass::HiZPass;
use crate::render::pass::ibl::sh_project_pass::ShProjectPass;
use crate::render::pass::main::main_pass::MainPass;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_layout::{RenderView, RenderViewsLayout};
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::physics_debug::physics_debug_pass::PhysicsDebugPass;
use crate::render::pass::selection::selection_pass::SelectionPass;
use crate::render::pass::shadows::shadows::Shadows;
use crate::render::pass::skinning::skinning_pass::SkinningPass;
use crate::render::pass::tlas_build::tlas_build_pass::TLASBuildPass;
use crate::render::pass::tlas_instances::tlas_instances_pass::TLASInstancesPass;
use crate::render::pass::tonemap::tonemap_pass::TonemapPass;
use crate::render::pass::ui::ui_frame::UiFrame;
use crate::render::pass::ui::ui_render_pass::UiPass;
use gpu::Queues;
use ray_tracing::RayTracing;
use crate::render::readback::entity_id_pick_reader::EntityIdPickReader;
use crate::render::readback::readback_pass::ReadbackPass;
use crate::render::readback::readbacks::Readbacks;
use crate::render::render_context::RenderContext;
use crate::render::render_graph::pass_graph::PassGraph;
use crate::render::render_graph::virtual_buffer::buffer_blueprint::BufferBlueprint;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::image_size::ImageSize;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::renderer_statistics::RenderStatistics;
use crate::render::state::render_state::RenderState;
use gpu::HDR_FORMAT;
use gpu::RenderTarget;
use gpu::BindingLayout;
use gpu::PipelineLayoutType;
use resource_store::ResourceBuffers;
use pipeline_store::PipelineStore;
use settings::RenderSettings;
use render_snapshot::{RenderEntityId, RenderSnapshot};
use index_allocator::{ArcUnwrapOrErr, ResourceId};
use gpu::ViewProjectionMatrix;
use gpu::{profile_cpu_meta, profile_cpu_zone};
use anyhow::Result;
use ash::vk::{
    AccelerationStructureInstanceKHR, AccessFlags, BufferUsageFlags, DeviceSize, Extent2D, Format,
    ImageLayout, ImageUsageFlags, PhysicalDevice, PipelineStageFlags, SubmitInfo,
};
use ash::{Device, Instance};
use glam::{Mat4, Vec2, Vec3};
use std::array::from_fn;
use std::collections::HashMap;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::info;

const JITTER_PHASE: u64 = 16;
const DRAW_BUCKET_COUNT: u32 = 3;

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
}

impl Render {
    pub fn create(
        instance: &Instance,
        device_context: &DeviceContext,
        limits: &RenderLimits,
        target: Arc<dyn RenderTarget>,
        resource_factories: Arc<ResourceFactories>,
        settings: RenderSettings,
        physical_device: PhysicalDevice,
        queues: &Queues,
        pipeline_store: Arc<PipelineStore>,
        ray_tracing: Option<Arc<RayTracing>>,
        binding_layout: Arc<BindingLayout>,
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
        let render_scale = settings.render_scale.value;
        let render_extent = Self::scaled_render_extent(target_extent, render_scale);

        let pass_graph_state = render_state
            .pass_graph_state
            .take()
            .expect("pass graph state");
        let mut pass_graph = PassGraph::new(pass_graph_state);

        let depth_image = pass_graph.create_image(
            "depth",
            ImageBlueprint::depth(ImageSize::render_full(), Format::D32_SFLOAT),
        );
        let normal_image = pass_graph.create_image(
            "normal",
            ImageBlueprint::color(ImageSize::render_full(), Format::R16G16B16A16_SFLOAT),
        );
        let velocity_image = pass_graph.create_image(
            "velocity",
            ImageBlueprint::color(ImageSize::render_full(), Format::R16G16_SFLOAT),
        );
        let entity_id_image = pass_graph.create_image(
            "entity_id",
            ImageBlueprint {
                usage: ImageUsageFlags::COLOR_ATTACHMENT
                    | ImageUsageFlags::TRANSFER_SRC
                    | ImageUsageFlags::TRANSFER_DST
                    | ImageUsageFlags::SAMPLED,
                ..ImageBlueprint::color(ImageSize::render_full(), Format::R32_UINT)
            },
        );
        let scene_color_image = pass_graph.create_image(
            "scene_color",
            ImageBlueprint::color(ImageSize::render_full(), scene_color_format),
        );
        let history_images: [VirtualImage; 2] = from_fn(|index| {
            pass_graph.create_image(
                if index == 0 { "history_a" } else { "history_b" },
                ImageBlueprint::storage(ImageSize::Target { pow: 0 }, scene_color_format),
            )
        });

        const BLOOM_MIPS: usize = 5;
        let bloom_image = pass_graph.create_image(
            "bloom",
            ImageBlueprint {
                image_view_description: ImageViewDescription {
                    level_count: BLOOM_MIPS as u32,
                    ..ImageViewDescription::default_2d_color()
                },
                ..ImageBlueprint::color(ImageSize::Render { pow: 1 }, scene_color_format)
            },
        );

        let hiz_base_width = (render_extent.width >> 1).max(1);
        let hiz_base_height = (render_extent.height >> 1).max(1);
        let hiz_mip_count = (32 - hiz_base_width.max(hiz_base_height).leading_zeros()).max(1);
        let hiz_image = pass_graph.create_image(
            "hiz",
            ImageBlueprint {
                image_view_description: ImageViewDescription {
                    level_count: hiz_mip_count,
                    ..ImageViewDescription::default_2d_color()
                },
                ..ImageBlueprint::storage(
                    ImageSize::Render { pow: 1 },
                    limits.hiz_limits.format.vulkan(),
                )
            },
        );
        let hiz_counter_buffer = pass_graph.create_buffer(
            "hiz_counter",
            BufferBlueprint::storage_dst(size_of::<u32>() as DeviceSize),
        );
        let brdf_lut_physical = render_state
            .image_scope
            .get_physical_image(render_state.brdf_lut_image);
        let brdf_lut_descriptor = render_state
            .bindless
            .graph_textures
            .acquire_image(brdf_lut_physical.image_view);
        let brdf_lut_image = pass_graph.import_image(
            "brdf_lut",
            brdf_lut_physical.image,
            brdf_lut_physical.image_view,
            brdf_lut_physical.extent,
            brdf_lut_physical.format,
            brdf_lut_physical.subresource_range,
            brdf_lut_descriptor,
        );
        let brdf_lut_main_descriptor = brdf_lut_physical.descriptors.full.unwrap_or(ResourceId::from(0));
        let sh_physical = render_state
            .image_scope
            .get_physical_image(render_state.sh_image);
        let sh_descriptor = render_state
            .bindless
            .graph_textures
            .acquire_image(sh_physical.image_view);
        let sh_image = pass_graph.import_image(
            "sh",
            sh_physical.image,
            sh_physical.image_view,
            sh_physical.extent,
            sh_physical.format,
            sh_physical.subresource_range,
            sh_descriptor,
        );
        let target_image = pass_graph.import_image_placeholder("render_target");

        let scene_buffer = pass_graph.import_buffer_placeholder("scene");
        let entity_buffer = pass_graph.import_buffer_placeholder("entity");
        let main_culling_views_buffer = pass_graph.import_buffer_placeholder("main_culling_views");
        let main_cull_requests_buffer = pass_graph.import_buffer_placeholder("main_cull_requests");
        let cascade_cull_requests_buffer = pass_graph.import_buffer_placeholder("cascade_cull_requests");
        let physics_debug_vertex_buffer = pass_graph.import_buffer_placeholder("physics_debug_vertex");
        let skinning_instance_buffer = pass_graph.import_buffer_placeholder("skinning_instance");
        let ui_index_buffer = pass_graph.import_buffer_placeholder("ui_index");
        let ui_vertex_buffer = pass_graph.import_buffer_placeholder("ui_vertex");

        let opaque_capacity = limits.resource_limits.max_draw_calls;
        let transparent_capacity = limits.resource_limits.max_transparent_draw_calls;
        let pool_capacity = 2 * opaque_capacity + 2 * transparent_capacity;

        let draw_pool = DrawPool {
            indirect: pass_graph.create_buffer(
                "draw_indirect_pool",
                BufferBlueprint::indirect((size_of::<IndirectGPU>() * pool_capacity as usize) as DeviceSize),
            ),
            draw_count: pass_graph.create_buffer(
                "draw_count_pool",
                BufferBlueprint::indirect_count((size_of::<u32>() * DRAW_BUCKET_COUNT as usize) as DeviceSize),
            ),
            draw_data: pass_graph.create_buffer(
                "draw_data_pool",
                BufferBlueprint::storage((size_of::<DrawDataGPU>() * pool_capacity as usize) as DeviceSize),
            ),
        };

        let main_bucket = DrawBucket { count_index: 0, draw_offset: 0, capacity: opaque_capacity };
        let transparent_bucket = DrawBucket { count_index: 1, draw_offset: opaque_capacity, capacity: transparent_capacity };
        let transparent_sorted_bucket = DrawBucket { count_index: 1, draw_offset: opaque_capacity + transparent_capacity, capacity: transparent_capacity };
        let shadow_bucket = DrawBucket { count_index: 2, draw_offset: opaque_capacity + 2 * transparent_capacity, capacity: opaque_capacity };

        let bone_transform = pass_graph.create_buffer(
            "bone_transform",
            BufferBlueprint::storage_dst(
                (size_of::<BoneTransformGPU>() * limits.resource_limits.max_bone_transforms as usize) as DeviceSize,
            ),
        );

        let pass_resources = PassResources {
            render_context: &render_context,
            pipeline_provider: &pipeline_store.pipeline_provider,
            compute_pipeline_provider: &pipeline_store.compute_pipeline_provider,
            pipeline_layout_registry: &binding_layout.pipeline_layout_registry,
        };

        let pick_reader = Arc::new(EntityIdPickReader::create(entity_id_image));
        let readbacks = Arc::new(Readbacks::new(
            &resource_factories.buffer_factory,
            vec![pick_reader.clone()],
            limits.frames_in_flight,
        )?);

        let ray_tracing_graph = if ray_tracing.is_some() {
            let blas = pass_graph.import_acceleration_structure();
            let tlas = pass_graph.import_acceleration_structure();

            let tlas_instances = pass_graph.create_buffer(
                "tlas_instances",
                BufferBlueprint::new(
                    limits.resource_limits.max_draw_calls as DeviceSize
                        * size_of::<AccelerationStructureInstanceKHR>() as DeviceSize,
                    BufferUsageFlags::STORAGE_BUFFER
                        | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
                ),
            );

            Some((blas, tlas, tlas_instances))
        } else {
            None
        };

        if let (Some(ray_tracing), Some((blas, _, _))) = (&ray_tracing, ray_tracing_graph) {
            pass_graph.add_pass(BLASBuildPass::create(ray_tracing.clone(), blas), &profiler);
        }

        pass_graph.add_pass(
            BrdfLutPass::create(
                Format::R16G16_SFLOAT,
                &pipeline_store.pipeline_provider,
                &binding_layout.pipeline_layout_registry,
                brdf_lut_image,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            FrameStagingPass::create(scene_buffer, entity_buffer, main_culling_views_buffer),
            &profiler,
        );
        pass_graph.add_pass(
            CullingIndirectPass::create(
                &pass_resources,

                limits.frames_in_flight,
                &resource_factories,
                "main_culling_indirect",
                MAIN_CULLING_META_NAME,
                1,
                false,
                scene_buffer,
                entity_buffer,
                main_culling_views_buffer,
                draw_pool,
                vec![
                    CullRequest { accept_mask: MaterialGPU::FLAG_ALPHA_OPAQUE | MaterialGPU::FLAG_ALPHA_MASK, bucket: main_bucket },
                    CullRequest { accept_mask: MaterialGPU::FLAG_ALPHA_BLEND, bucket: transparent_bucket },
                ],
                main_cull_requests_buffer,
            )?,
            &profiler,
        );

        if let (Some(ray_tracing), Some((blas, tlas, tlas_instances))) =
            (&ray_tracing, ray_tracing_graph)
        {
            pass_graph.add_pass(
                TLASInstancesPass::create(
                    &pass_resources,
                    ray_tracing.clone(),
                    entity_buffer,
                    tlas_instances,
                )?,
                &profiler,
            );
            pass_graph.add_pass(
                TLASBuildPass::create(ray_tracing.clone(), tlas_instances, blas, tlas),
                &profiler,
            );
        }

        pass_graph.add_pass(
            ShProjectPass::create(
                scene_color_format,
                &pipeline_store.pipeline_provider,
                &binding_layout.pipeline_layout_registry,
                scene_buffer,
                sh_image,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            SkinningPass::create(
                &pass_resources,
                skinning_instance_buffer,
                bone_transform,
            )?,
            &profiler,
        );
        let rt_ao = ray_tracing_graph.is_some() && settings.rt_ao.value;

        pass_graph.add_pass(
            DepthPrepass::create(
                &pass_resources,
                depth_image,
                normal_image,
                Format::R16G16B16A16_SFLOAT,
                velocity_image,
                Format::R16G16_SFLOAT,
                scene_buffer,
                entity_buffer,
                draw_pool,
                main_bucket,
                bone_transform,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            HiZPass::create(
                &pass_resources,
                depth_image,
                hiz_image,
                hiz_counter_buffer,
                hiz_mip_count,
            )?,
            &profiler,
        );
        let ao = Ao::build(
            &mut pass_graph,
            &pass_resources,
            &profiler,
            depth_image,
            normal_image,
            velocity_image,
            scene_buffer,
            rt_ao,
            ray_tracing_graph.map(|(_, tlas, _)| tlas),
        )?;
        let shadows = Shadows::build(
            &mut pass_graph,
            &pass_resources,
            &profiler,
            &resource_factories,
            &settings,
            ray_tracing.is_some(),
            limits,
            depth_image,
            normal_image,
            velocity_image,
            scene_buffer,
            entity_buffer,
            bone_transform,
            draw_pool,
            shadow_bucket,
            cascade_cull_requests_buffer,
            ao.guide[0],
            ao.guide[1],
            ray_tracing_graph.map(|(_, tlas, _)| tlas),
        )?;
        pass_graph.add_pass(
            EnvironmentPass::create(
                &pass_resources,
                scene_color_format,
                scene_color_image,
                depth_image,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            MainPass::create(
                &pass_resources,
                scene_color_format,
                scene_color_image,
                entity_id_image,
                depth_image,
                shadows.history[0],
                shadows.history[1],
                shadows.colored,
                ao.history[0],
                ao.history[1],
                sh_image,
                brdf_lut_image,
                brdf_lut_main_descriptor.inner,
                scene_buffer,
                entity_buffer,
                draw_pool,
                main_bucket,
                bone_transform,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            DrawSortPass::create(
                &pass_resources,
                &resource_factories,
                limits.frames_in_flight,
                limits.resource_limits.max_sorted_draw_calls,
                draw_pool,
                transparent_bucket,
                transparent_sorted_bucket,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            TransparentPass::create(
                &pass_resources,
                scene_color_format,
                scene_color_image,
                depth_image,
                sh_image,
                brdf_lut_main_descriptor.inner,
                scene_buffer,
                entity_buffer,
                draw_pool,
                transparent_sorted_bucket,
                bone_transform,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            TransparentEntityIdPass::create(
                &pass_resources,
                entity_id_image,
                depth_image,
                scene_buffer,
                entity_buffer,
                draw_pool,
                transparent_sorted_bucket,
                bone_transform,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            AccumulatePass::create(
                &pass_resources,
                scene_color_image,
                velocity_image,
                history_images[0],
                history_images[1],
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            BloomDownsamplePass::create(
                &pass_resources,
                scene_color_format,
                scene_color_image,
                None,
                bloom_image,
                0,
                true,
            )?,
            &profiler,
        );
        for index in 1..BLOOM_MIPS {
            pass_graph.add_pass(
                BloomDownsamplePass::create(
                    &pass_resources,
                    scene_color_format,
                    bloom_image,
                    Some((index - 1) as u32),
                    bloom_image,
                    index as u32,
                    false,
                )?,
                &profiler,
            );
        }
        for index in (0..BLOOM_MIPS - 1).rev() {
            pass_graph.add_pass(
                BloomUpsamplePass::create(
                    &pass_resources,
                    scene_color_format,
                    bloom_image,
                    (index + 1) as u32,
                    index as u32,
                )?,
                &profiler,
            );
        }
        pass_graph.add_pass(
            TonemapPass::create(
                &pass_resources,
                color_format,
                scene_color_image,
                history_images[0],
                history_images[1],
                bloom_image,
                target_image,
                color_format == HDR_FORMAT,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            DebugLayerPass::create(
                &pass_resources,
                color_format,
                velocity_image,
                normal_image,
                ao.raw,
                sh_image,
                hiz_image,
                shadows.history[0],
                shadows.history[1],
                shadows.colored,
                target_image,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            SelectionPass::create(
                &pass_resources,
                color_format,
                target_image,
                entity_id_image,
                [1.0, 0.5, 0.0, 0.15],
                pick_reader.clone(),
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            PhysicsDebugPass::create(
                &pass_resources,
                color_format,
                target_image,
                physics_debug_vertex_buffer,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            UiPass::create(
                &pass_resources,
                ui_index_buffer,
                ui_vertex_buffer,
                color_format,
                target_image,
            )?,
            &profiler,
        );
        pass_graph.add_pass(ReadbackPass::new(readbacks.clone()), &profiler);

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
        })
    }

    pub fn render_frame(
        &mut self,
        device_context: &DeviceContext,
        limits: &RenderLimits,
        resource_buffers: &ResourceBuffers,
        render_snapshot: Arc<RenderSnapshot>,
        render_settings: RenderSettings,
        ui_frame: UiFrame,
    ) -> Result<()> {
        let frame_index = self.render_context.next_frame_index();
        let frame_context = self.render_context.get_frame(frame_index)?;

        unsafe {
            device_context
                .device
                .wait_for_fences(&[frame_context.fence], true, u64::MAX)?
        };

        self.render_state.bindless.update();

        let Some(image_index) = self
            .target
            .acquire_next_image(frame_context.acquire_semaphore)?
        else {
            return Ok(());
        };

        self.profiler.begin_frame(frame_index);

        self.readbacks.sync(frame_index);

        let skinned_entities = render_snapshot
            .entities
            .iter()
            .filter(|entity| entity.animation.is_some())
            .count() as u32;
        profile_cpu_meta!(
            &self.profiler,
            "world.entities",
            render_snapshot.entities.len() as u32
        );
        profile_cpu_meta!(&self.profiler, "world.skinned_entities", skinned_entities);
        profile_cpu_meta!(
            &self.profiler,
            "world.debug_lines",
            render_snapshot.debug_lines.len() as u32
        );

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

        self.render_state
            .cpu_to_gpu_allocator
            .begin_frame(frame_index);
        self.pass_graph.begin_transient_buffers_frame(frame_index);

        let target_extent = self.target.extent();
        let mut render_views_layout =
            self.build_render_views_layout(&render_settings, target_extent, &limits, &render_snapshot);

        let current_main_view_projection = render_views_layout.main.view_projection;
        render_views_layout.main.previous_view_projection = self
            .previous_view_projection
            .unwrap_or(current_main_view_projection);

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
            &limits,
            render_settings,
            &render_views_layout,
            render_snapshot.clone(),
            previous_transforms,
            ui_frame,
        );

        let frame_number = self.frame_counter.load(Ordering::Relaxed);
        let history_write_index = (frame_number & 1) as u32;
        let history_valid = frame_number != 0;

        let render_pass_context = PassContext::create(
            &device_context,
            &self.render_context,
            &limits,
            render_settings,
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
        profile_cpu_meta!(
            &self.profiler,
            "render.cpu_to_gpu.capacity",
            cpu_to_gpu.capacity
        );

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

        self.target
            .present(&device_context.queues, image_index, present_semaphore)?;

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
            pass_graph.run(&frame_data_context, &pass_context, profiler, allocator)?;
        });

        profiler.extract_queries(command_buffer, pass_context.frame_index);

        pass_context.end_command_recording()?;

        Ok(())
    }

    fn build_render_views_layout(
        &self,
        render_settings: &RenderSettings,
        extent: Extent2D,
        limits: &RenderLimits,
        render_snapshot: &RenderSnapshot,
    ) -> RenderViewsLayout {
        let aspect_ratio = extent.width as f32 / extent.height as f32;

        let camera_view = render_snapshot.camera.view();
        let camera_projection = render_snapshot.camera.projection(aspect_ratio);

        let tan_half_fov_x = 1.0 / camera_projection.value.x_axis.x;
        let tan_half_fov_y = 1.0 / camera_projection.value.y_axis.y;

        let view_projection =
            ViewProjectionMatrix::from_view_projection(&camera_view, &camera_projection)
                .vulkan_corrected();

        let render_width = self.render_extent.width.max(1) as f32;
        let render_height = self.render_extent.height.max(1) as f32;

        let jittered_view_projection = if render_settings.fsr_enabled.value {
            let jitter_index =
                (self.frame_counter.load(Ordering::Relaxed) % JITTER_PHASE) as u32 + 1;
            let jitter_ndc_x = (Self::halton(jitter_index, 2) - 0.5) * 2.0 / render_width;
            let jitter_ndc_y = (Self::halton(jitter_index, 3) - 0.5) * 2.0 / render_height;

            ViewProjectionMatrix {
                value: Mat4::from_translation(Vec3::new(jitter_ndc_x, jitter_ndc_y, 0.0))
                    * view_projection.value,
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
        limits: &RenderLimits,
        resource_factories: Arc<ResourceFactories>,
        settings: RenderSettings,
        physical_device: PhysicalDevice,
        binding_layout: Arc<BindingLayout>,
        pipeline_store: Arc<PipelineStore>,
        ray_tracing: Option<Arc<RayTracing>>,
    ) -> Result<Self> {
        let target = self.target.clone();
        let profiler = self.profiler.clone();
        let frame_counter = self.frame_counter.clone();
        let hdr = settings.hdr.value && target.hdr_supported();
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
            pipeline_store,
            ray_tracing,
            binding_layout,
            profiler.clone(),
            frame_counter,
            render_state,
        )?;

        profiler.flush_pending_provider_destroy(&resource_factories)?;

        Ok(render)
    }

    fn destroy_inner(
        self,
        device: &Device,
        resource_factories: &ResourceFactories,
    ) -> Result<RenderState> {
        let Self {
            render_context,
            pass_graph,
            mut render_state,
            readbacks,
            ..
        } = self;

        render_state.pass_graph_state = Some(pass_graph.destroy(&resource_factories)?);

        readbacks
            .try_unwrap()?
            .destroy(&resource_factories.buffer_factory)?;

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
