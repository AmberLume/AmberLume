use gpu_data::MaterialGPU;
use crate::limits::RenderLimits;
use gpu::{profile_gpu_zone, RayTracingContext};
use gpu::FrameProfiler;
use gpu::DeviceContext;
use gpu::VulkanContext;
use gpu::ImageViewDescription;
use gpu::ResourceFactories;
use crate::render::pass::ao::Ao;
use crate::render::pass::blas_build::blas_build_pass::BLASBuildPass;
use crate::render::pass::bloom::bloom_downsample_pass::BloomDownsamplePass;
use crate::render::pass::bloom::bloom_upsample_pass::BloomUpsamplePass;
use crate::render::pass::brdf_lut::brdf_lut_pass::BrdfLutPass;
use crate::render::pass::culling_indirect::cull_request::CullRequest;
use render_graph::DrawBucket;
use crate::render::pass::draw_pool::DrawPool;
use crate::render::pass::culling_indirect::culling_indirect_pass::CullingIndirectPass;
use statistics::CullingIndirectRequestStatisticsGPU;
use crate::render::pass::frame_staging::frame_staging_pass::FrameStagingPass;
use crate::render::pass::draw_sort::draw_sort_pass::DrawSortPass;
use crate::render::pass::transparent::transparent_pass::TransparentPass;
use crate::render::pass::transparent_entity_id::transparent_entity_id_pass::TransparentEntityIdPass;
use crate::render::pass::debug_layer::debug_layer_pass::DebugLayerPass;
use crate::render::pass::depth::depth_prepass::DepthPrepass;
use crate::render::pass::environment::environment_pass::EnvironmentPass;
use crate::render::pass::fsr2::accumulate_pass::AccumulatePass;
use crate::render::pass::hiz::hiz_pass::HiZPass;
use crate::render::pass::ibl::sh_project_pass::ShProjectPass;
use crate::render::pass::main::main_pass::MainPass;
use render_graph::FrameContext;
use crate::render::pass::pass_layout::{RenderView, RenderViewsLayout};
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::resource_buffer_handles::ResourceBufferHandles;
use crate::render::pass::physics_debug::physics_debug_pass::PhysicsDebugPass;
use crate::render::pass::selection::selection_pass::SelectionPass;
use crate::render::pass::selection_mask::selection_mask_pass::SelectionMaskPass;
use crate::render::pass::shadows::shadows::Shadows;
use crate::render::pass::skinning::skinning_pass::SkinningPass;
use crate::render::pass::terrain_generate::terrain_generate_pass::TerrainGeneratePass;
use crate::render::pass::terrain_points::terrain_points_pass::TerrainPointsPass;
use crate::render::pass::terrain_stitch::terrain_stitch_pass::TerrainStitchPass;
use crate::render::pass::tlas_build::tlas_build_pass::TLASBuildPass;
use crate::render::pass::tlas_instances::tlas_instances_pass::TLASInstancesPass;
use crate::render::pass::tonemap::tonemap_pass::TonemapPass;
use ui::UiFrame;
use crate::render::pass::ui::ui_render_pass::UiPass;
use gpu::Queues;
use ray_tracing::RayTracing;
use crate::render::render_context::RenderContext;
use render_graph::PassGraph;
use render_graph::ImageBlueprint;
use render_graph::ImageSize;
use statistics::CascadeStatisticsGPU;
use statistics::DrawSortStatisticsGPU;
use render_graph::VirtualData;
use bytemuck::Pod;
use render_graph::VirtualReadback;
use crate::render::frame_data::picked_entity_gpu::PickedEntityGPU;
use render_graph::VirtualImage;
use statistics::RenderStatistics;
use crate::render::state::render_state::RenderState;
use gpu::HDR_FORMAT;
use gpu::RenderTarget;
use gpu::BindingLayout;
use gpu::PipelineLayoutType;
use resource_residency::ResourceProvider;
use resource_store::MeshBackend;
use resource_store::ResourceBuffers;
use pipeline_store::PipelineStore;
use settings::PresentMode;
use settings::RenderSettings;
use crate::render::frame_data::terrain_frame::TerrainFrame;
use render_snapshot::{RenderEntityId, RenderSnapshot};
use index_allocator::ResourceId;
use gpu::ViewProjectionMatrix;
use gpu::{profile_cpu_meta, profile_cpu_zone};
use anyhow::Result;
use ash::vk::{
    AccessFlags, DeviceSize, Extent2D, Format,
    ImageLayout, ImageUsageFlags, PhysicalDevice, PipelineStageFlags, PresentModeKHR, SubmitInfo,
};
use ash::{Device, Instance};
use glam::{Mat4, Vec2, Vec3};
use std::array::from_fn;
use std::collections::HashMap;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::info;

pub struct Render {
    pub target: Arc<dyn RenderTarget>,

    render_context: RenderContext,

    render_extent: Extent2D,

    target_image: VirtualImage,

    pass_graph: PassGraph,

    render_state: RenderState,
    binding_layout: Arc<BindingLayout>,

    profiler: Arc<FrameProfiler>,

    main_culling_statistics: VirtualReadback<CullingIndirectRequestStatisticsGPU>,
    cascade_culling_statistics: VirtualReadback<CullingIndirectRequestStatisticsGPU>,
    cascade_compute_statistics: VirtualReadback<CascadeStatisticsGPU>,
    draw_sort_statistics: VirtualReadback<DrawSortStatisticsGPU>,
    pub picked_entity: VirtualReadback<PickedEntityGPU>,

    render_settings: VirtualData<RenderSettings>,
    render_snapshot: VirtualData<RenderSnapshot>,
    render_views_layout: VirtualData<RenderViewsLayout>,
    previous_transforms_input: VirtualData<Vec<Mat4>>,
    ui_frame: VirtualData<UiFrame>,
    terrain_frame: VirtualData<TerrainFrame>,
    touched_meshes: VirtualData<Vec<ResourceId>>,
    ray_tracing_input: VirtualData<Arc<RayTracing>>,

    mesh_provider: Arc<ResourceProvider<MeshBackend>>,

    previous_view_projection: Option<ViewProjectionMatrix>,
    previous_transform_store: HashMap<RenderEntityId, Mat4>,

    frame_counter: Arc<AtomicU64>,
    created_frame: u64,
}

impl Render {
    pub fn create(
        instance: &Instance,
        device_context: &DeviceContext,
        ray_tracing_context: Option<&RayTracingContext>,
        limits: &RenderLimits,
        target: Arc<dyn RenderTarget>,
        resource_factories: Arc<ResourceFactories>,
        settings: RenderSettings,
        physical_device: PhysicalDevice,
        queues: &Queues,
        pipeline_store: Arc<PipelineStore>,
        binding_layout: Arc<BindingLayout>,
        resource_buffers: &ResourceBuffers,
        mesh_provider: Arc<ResourceProvider<MeshBackend>>,
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

        let render_settings = pass_graph.import_data::<RenderSettings>("render_settings");
        let render_snapshot = pass_graph.import_data::<RenderSnapshot>("render_snapshot");
        let render_views_layout = pass_graph.import_data::<RenderViewsLayout>("render_views_layout");
        let previous_transforms_input = pass_graph.import_data::<Vec<Mat4>>("previous_transforms");
        let ui_frame = pass_graph.import_data::<UiFrame>("ui_frame");
        let terrain_frame = pass_graph.import_data::<TerrainFrame>("terrain_frame");
        let ray_tracing_input = pass_graph.import_data::<Arc<RayTracing>>("ray_tracing");
        let touched_meshes = pass_graph.import_data::<Vec<ResourceId>>("touched_meshes");

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
                    | ImageUsageFlags::TRANSFER_DST
                    | ImageUsageFlags::SAMPLED,
                ..ImageBlueprint::color(ImageSize::render_full(), Format::R32_UINT)
            },
        );
        let selection_mask_image = pass_graph.create_image(
            "selection_mask",
            ImageBlueprint::storage(ImageSize::Render { pow: 3 }, Format::R32_UINT),
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
        let hiz_counter_buffer = pass_graph.create_device_buffer("hiz_counter", true);
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

        let scene_buffer = pass_graph.create_upload_buffer("scene", false);
        let entity_buffer = pass_graph.create_upload_buffer("entity", false);
        let entity_motion_buffer = pass_graph.create_upload_buffer("entity_motion", false);
        let entity_outline_buffer = pass_graph.create_upload_buffer("entity_outline", false);
        let main_culling_views_buffer = pass_graph.create_upload_buffer("main_culling_views", false);
        let main_cull_requests_buffer = pass_graph.create_upload_buffer("main_cull_requests", false);
        let cascade_cull_requests_buffer = pass_graph.create_upload_buffer("cascade_cull_requests", false);
        let physics_debug_vertex_buffer = pass_graph.create_upload_buffer("physics_debug_vertex", false);
        let skinning_instance_buffer = pass_graph.create_upload_buffer("skinning_instance", false);
        let terrain_generate_request_buffer = pass_graph.create_upload_buffer("terrain_generate_request", false);
        let terrain_height_buffer = pass_graph.create_upload_buffer("terrain_height", false);
        let terrain_stitch_request_buffer = pass_graph.create_upload_buffer("terrain_stitch_request", false);
        let terrain_edge_height_buffer = pass_graph.create_upload_buffer("terrain_edge_height", false);
        let terrain_chunk_buffer = pass_graph.create_upload_buffer("terrain_chunk", false);
        let ui_index_buffer = pass_graph.create_upload_buffer("ui_index", false);
        let ui_vertex_buffer = pass_graph.create_upload_buffer("ui_vertex", false);

        let opaque_capacity = limits.resource_limits.max_draw_calls;
        let transparent_capacity = limits.resource_limits.max_transparent_draw_calls;
        let pool_capacity = 2 * opaque_capacity + 2 * transparent_capacity;

        let draw_pool = DrawPool {
            indirect: pass_graph.create_device_buffer("draw_indirect_pool", false),
            draw_count: pass_graph.create_device_buffer("draw_count_pool", true),
            draw_data: pass_graph.create_device_buffer("draw_data_pool", false),

            capacity: pool_capacity,
        };

        let main_bucket = DrawBucket { count_index: 0, draw_offset: 0, capacity: opaque_capacity };
        let transparent_bucket = DrawBucket { count_index: 1, draw_offset: opaque_capacity, capacity: transparent_capacity };
        let transparent_sorted_bucket = DrawBucket { count_index: 1, draw_offset: opaque_capacity + transparent_capacity, capacity: transparent_capacity };
        let shadow_bucket = DrawBucket { count_index: 2, draw_offset: opaque_capacity + 2 * transparent_capacity, capacity: opaque_capacity };

        let bone_transform = pass_graph.create_device_buffer("bone_transform", false);

        let resource_buffer_handles = ResourceBufferHandles::import(&mut pass_graph, resource_buffers);

        let pass_resources = PassResources {
            render_context: &render_context,
            resource_buffer_handles,
            pipeline_provider: &pipeline_store.pipeline_provider,
            compute_pipeline_provider: &pipeline_store.compute_pipeline_provider,
            pipeline_layout_registry: &binding_layout.pipeline_layout_registry,
        };

        let main_culling_statistics = pass_graph.create_readback::<CullingIndirectRequestStatisticsGPU>(
            &resource_factories.buffer_factory,
            "main_culling_statistics",
            2,
            limits.frames_in_flight,
        )?;

        let cascade_culling_statistics = pass_graph.create_readback::<CullingIndirectRequestStatisticsGPU>(
            &resource_factories.buffer_factory,
            "cascade_culling_statistics",
            1,
            limits.frames_in_flight,
        )?;

        let cascade_compute_statistics = pass_graph.create_readback::<CascadeStatisticsGPU>(
            &resource_factories.buffer_factory,
            "cascade_compute_statistics",
            limits.shadow_map_limits.cascade_count,
            limits.frames_in_flight,
        )?;

        let picked_entity = pass_graph.create_readback::<PickedEntityGPU>(
            &resource_factories.buffer_factory,
            "picked_entity",
            1,
            limits.frames_in_flight,
        )?;

        let draw_sort_statistics = pass_graph.create_readback::<DrawSortStatisticsGPU>(
            &resource_factories.buffer_factory,
            "draw_sort_statistics",
            1,
            limits.frames_in_flight,
        )?;

        let ray_tracing_graph = if let Some(ray_tracing_context) = ray_tracing_context {
            let properties = ray_tracing_context.properties;

            let blas = pass_graph.import_acceleration_structure();
            let tlas = pass_graph.import_acceleration_structure();

            let blas_addresses = pass_graph.create_upload_buffer("blas_addresses", false);
            let blas_scratch = pass_graph.create_scratch_buffer("blas_scratch", properties.min_scratch_offset_alignment as DeviceSize);

            let tlas_instances = pass_graph.create_device_buffer("tlas_instances", false);

            Some((blas, tlas, blas_addresses, blas_scratch, tlas_instances))
        } else {
            None
        };

        pass_graph.add_pass(
            TerrainGeneratePass::create(
                &pass_resources,
                terrain_generate_request_buffer,
                terrain_height_buffer,
                terrain_frame,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            TerrainStitchPass::create(
                &pass_resources,
                terrain_stitch_request_buffer,
                terrain_edge_height_buffer,
                terrain_frame,
            )?,
            &profiler,
        );

        if let Some((blas, _, blas_addresses, blas_scratch, _)) = ray_tracing_graph {
            pass_graph.add_pass(
                BLASBuildPass::create(
                    ray_tracing_input,
                    render_snapshot,
                    touched_meshes,
                    blas,
                    blas_addresses,
                    blas_scratch,
                    resource_buffer_handles.mesh_vertex_buffer,
                    resource_buffer_handles.index_buffer,
                    mesh_provider.clone(),
                ),
                &profiler,
            );
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
            FrameStagingPass::create(
                scene_buffer,
                entity_buffer,
                entity_motion_buffer,
                entity_outline_buffer,
                main_culling_views_buffer,
                render_snapshot,
                render_views_layout,
                previous_transforms_input,
            ),
            &profiler,
        );
        pass_graph.add_pass(
            CullingIndirectPass::create(
                &pass_resources,
                "main_culling_indirect",
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
                render_snapshot,
                main_culling_statistics,
            )?,
            &profiler,
        );

        if let Some((blas, tlas, blas_addresses, _, tlas_instances)) = ray_tracing_graph {
            pass_graph.add_pass(
                TLASInstancesPass::create(
                    &pass_resources,
                    entity_buffer,
                    blas_addresses,
                    tlas_instances,
                    render_snapshot,
                )?,
                &profiler,
            );
            pass_graph.add_pass(
                TLASBuildPass::create(ray_tracing_input, tlas_instances, blas, tlas, render_snapshot),
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
                limits.resource_limits.max_bone_transforms,
                render_snapshot,
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
                entity_motion_buffer,
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
            settings.ao_spatial.value,
            ray_tracing_graph.map(|(_, tlas, _, _, _)| tlas),
            render_settings,
        )?;
        let shadows = Shadows::build(
            &mut pass_graph,
            &pass_resources,
            &profiler,
            &settings,
            ray_tracing_graph.is_some(),
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
            ray_tracing_graph.map(|(_, tlas, _, _, _)| tlas),
            render_settings,
            render_snapshot,
            cascade_culling_statistics,
            cascade_compute_statistics,
        )?;
        pass_graph.add_pass(
            EnvironmentPass::create(
                &pass_resources,
                scene_color_format,
                scene_color_image,
                Format::R16G16_SFLOAT,
                velocity_image,
                depth_image,
                scene_buffer,
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
                picked_entity,
                render_settings,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            DrawSortPass::create(
                &pass_resources,
                limits.resource_limits.max_sorted_draw_calls,
                draw_pool,
                transparent_bucket,
                transparent_sorted_bucket,
                draw_sort_statistics,
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
            TerrainPointsPass::create(
                &pass_resources,
                scene_color_format,
                scene_color_image,
                depth_image,
                terrain_chunk_buffer,
                scene_buffer,
                terrain_frame,
                render_settings,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            TransparentEntityIdPass::create(
                &pass_resources,
                entity_id_image,
                Format::R16G16_SFLOAT,
                velocity_image,
                depth_image,
                scene_buffer,
                entity_buffer,
                entity_motion_buffer,
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
                render_settings,
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
                render_settings,
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
                    render_settings,
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
                    render_settings,
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
                render_settings,
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
                ao.history[0],
                ao.history[1],
                target_image,
                scene_buffer,
                render_settings,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            SelectionMaskPass::create(
                &pass_resources,
                entity_id_image,
                selection_mask_image,
                entity_outline_buffer,
                render_snapshot,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            SelectionPass::create(
                &pass_resources,
                color_format,
                target_image,
                entity_id_image,
                selection_mask_image,
                entity_outline_buffer,
                scene_buffer,
                render_snapshot,
            )?,
            &profiler,
        );
        pass_graph.add_pass(
            PhysicsDebugPass::create(
                &pass_resources,
                color_format,
                target_image,
                physics_debug_vertex_buffer,
                scene_buffer,
                render_snapshot,
                render_settings,
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
                ui_frame,
            )?,
            &profiler,
        );

        pass_graph.build(
            target_extent,
            render_extent,
            &resource_factories,
            &render_state.bindless.graph_textures,
            &render_state.bindless.storage_images,
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

            main_culling_statistics,
            cascade_culling_statistics,
            cascade_compute_statistics,
            draw_sort_statistics,
            picked_entity,

            render_settings,
            render_snapshot,
            render_views_layout,
            previous_transforms_input,
            ui_frame,
            terrain_frame,
            touched_meshes,
            ray_tracing_input,

            mesh_provider,

            previous_view_projection: None,
            previous_transform_store: HashMap::new(),

            created_frame: frame_counter.load(Ordering::Relaxed),
            frame_counter,
        })
    }

    pub fn render_frame(
        &mut self,
        device_context: &DeviceContext,
        limits: &RenderLimits,
        render_snapshot: RenderSnapshot,
        render_settings: RenderSettings,
        ui_frame: UiFrame,
        terrain_frame: TerrainFrame,
        ray_tracing: Option<&Arc<RayTracing>>,
    ) -> Result<()> {
        let frame_index = self.render_context.next_frame_index();
        let frame_resources = self.render_context.get_frame(frame_index)?;

        unsafe {
            device_context
                .device
                .wait_for_fences(&[frame_resources.fence], true, u64::MAX)?
        };

        self.render_state.bindless.update();

        let Some(image_index) = self
            .target
            .acquire_next_image(frame_resources.acquire_semaphore)?
        else {
            return Ok(());
        };

        self.profiler.begin_frame(frame_index);

        self.pass_graph.begin_readback_frame(frame_index);

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

        self.pass_graph.begin_buffers_frame(frame_index)?;

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
                self.previous_transform_store
                    .get(&entity.id)
                    .copied()
                    .unwrap_or(entity.transform_matrix)
            })
            .collect();


        self.pass_graph.set_input(self.render_settings, render_settings);
        self.previous_transform_store = render_snapshot
            .entities
            .iter()
            .filter(|entity| entity.id != RenderEntityId::STATIC)
            .map(|entity| (entity.id, entity.transform_matrix))
            .collect();

        self.pass_graph.set_input(self.render_snapshot, render_snapshot);
        self.pass_graph.set_input(self.previous_transforms_input, previous_transforms);
        self.pass_graph.set_input(self.ui_frame, ui_frame);
        let touched_meshes = terrain_frame
            .stitch_requests
            .iter()
            .map(|terrain_stitch_request| terrain_stitch_request.mesh_id)
            .collect::<Vec<_>>();

        self.pass_graph.set_input(self.touched_meshes, touched_meshes);
        self.pass_graph.set_input(self.terrain_frame, terrain_frame);

        if let Some(ray_tracing) = ray_tracing {
            self.pass_graph.set_input(self.ray_tracing_input, ray_tracing.clone());
        }

        self.pass_graph.set_input(self.render_views_layout, render_views_layout);

        let frame_number = self.frame_counter.load(Ordering::Relaxed);
        let history_write_index = (frame_number & 1) as u32;
        let history_valid = frame_number != self.created_frame;

        let frame_context = FrameContext::create(
            &device_context,
            &frame_resources.command_recording,
            ray_tracing.map(|ray_tracing| &ray_tracing.context),
            target_image,
            frame_index,
            frame_number as u32,
            history_write_index,
            history_valid,
        );

        profile_cpu_zone!(&self.profiler, "render.collect_commands", {
            Self::collect_render_commands(
                &frame_context,
                &self.binding_layout,
                &self.profiler,
                &mut self.pass_graph,
            )?;
        });

        let present_semaphore = self.target.get_present_semaphore(image_index)?;

        let wait_semaphores = [frame_resources.acquire_semaphore];
        let signal_semaphores = [present_semaphore];
        let wait_stages = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit_info = SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(slice::from_ref(
                &frame_resources.command_recording.command_buffer,
            ))
            .signal_semaphores(&signal_semaphores);

        unsafe { device_context.device.reset_fences(&[frame_resources.fence])? };

        device_context
            .queues
            .submit_graphics(submit_info, frame_resources.fence)?;

        self.target
            .present(&device_context.queues, image_index, present_semaphore)?;

        self.profiler.end_frame();

        self.previous_view_projection = Some(current_main_view_projection);

        Ok(())
    }

    fn collect_render_commands(
        pass_context: &FrameContext,
        binding_layout: &BindingLayout,
        profiler: &FrameProfiler,
        pass_graph: &mut PassGraph,
    ) -> Result<()> {
        let command_buffer = pass_context.command_buffer();

        pass_context.begin_command_recording()?;

        binding_layout.descriptor_set_manager.bind(
            command_buffer,
            binding_layout
                .pipeline_layout_registry
                .get(PipelineLayoutType::General),
        );

        profile_gpu_zone!(profiler, command_buffer, "render.total_dispatch", {
            pass_graph.run(&pass_context, profiler)?;
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

        let display_scale = extent.width.max(1) as f32 / render_width;
        let jitter_phase = ((8.0 * display_scale * display_scale).ceil() as u64).max(1);

        let jitter = if render_settings.fsr_enabled.value {
            let jitter_index = (self.frame_counter.load(Ordering::Relaxed) % jitter_phase) as u32 + 1;

            [
                Self::halton(jitter_index, 2) - 0.5,
                Self::halton(jitter_index, 3) - 0.5,
            ]
        } else {
            [0.0; 2]
        };

        let jittered_view_projection = if render_settings.fsr_enabled.value {
            ViewProjectionMatrix {
                value: Mat4::from_translation(Vec3::new(
                    jitter[0] * 2.0 / render_width,
                    jitter[1] * 2.0 / render_height,
                    0.0,
                )) * view_projection.value,
            }
        } else {
            view_projection
        };

        let mip_bias = if render_settings.fsr_enabled.value {
            (1.0 / display_scale).log2() - 1.0
        } else {
            0.0
        };

        RenderViewsLayout {
            main: RenderView {
                view_projection,
                view: camera_view,

                ndc_to_view_mul: Vec2::new(2.0 * tan_half_fov_x, -2.0 * tan_half_fov_y),
                ndc_to_view_add: Vec2::new(-tan_half_fov_x, tan_half_fov_y),

                previous_view_projection: view_projection,
                jittered_view_projection,

                jitter,

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
            hdr_supported: self.target.hdr_supported(),

            main_culling: self.pass_graph.readback_values(self.main_culling_statistics).map(<[_]>::to_vec),
            cascade_culling: self.pass_graph.readback_values(self.cascade_culling_statistics).map(<[_]>::to_vec),
            cascade_compute: self.pass_graph.readback_values(self.cascade_compute_statistics).map(<[_]>::to_vec),
            draw_sort: self.pass_graph.readback_value(self.draw_sort_statistics).copied(),
        }
    }

    pub fn readback_value<T: Pod>(&self, readback: VirtualReadback<T>) -> Option<&T> {
        self.pass_graph.readback_value(readback)
    }

    fn scaled_render_extent(target_extent: Extent2D, render_scale: f32) -> Extent2D {
        let scale = render_scale.clamp(0.1, 1.0);

        Extent2D {
            width: ((target_extent.width as f32 * scale).round() as u32).max(1),
            height: ((target_extent.height as f32 * scale).round() as u32).max(1),
        }
    }

    pub fn present_mode(settings: &RenderSettings) -> PresentModeKHR {
        match PresentMode::from_index(settings.present_mode.value) {
            PresentMode::Immediate => PresentModeKHR::IMMEDIATE,
            PresentMode::Mailbox => PresentModeKHR::MAILBOX,
            PresentMode::Fifo => PresentModeKHR::FIFO,
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
        ray_tracing_context: Option<&RayTracingContext>,
        limits: &RenderLimits,
        resource_factories: Arc<ResourceFactories>,
        settings: RenderSettings,
        physical_device: PhysicalDevice,
        binding_layout: Arc<BindingLayout>,
        pipeline_store: Arc<PipelineStore>,
        resource_buffers: &ResourceBuffers,
    ) -> Result<Self> {
        let target = self.target.clone();
        let mesh_provider = self.mesh_provider.clone();
        let profiler = self.profiler.clone();
        let frame_counter = self.frame_counter.clone();
        let hdr = settings.hdr.value && target.hdr_supported();
        target.invalidate(vulkan_context, device_context, hdr, Self::present_mode(&settings))?;

        let render_state = self.destroy_inner(&device_context.device, &resource_factories)?;

        let render = Self::create(
            instance,
            device_context,
            ray_tracing_context,
            limits,
            target,
            resource_factories.clone(),
            settings,
            physical_device,
            &device_context.queues,
            pipeline_store,
            binding_layout,
            resource_buffers,
            mesh_provider,
            profiler.clone(),
            frame_counter,
            render_state,
        )?;

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
