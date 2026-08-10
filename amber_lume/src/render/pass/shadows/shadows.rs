use render_graph::VirtualData;
use render_snapshot::RenderSnapshot;
use gpu_data::MaterialGPU;
use anyhow::Result;
use ash::vk::{DeviceSize, Format};
use std::array::from_fn;
use crate::limits::RenderLimits;
use gpu::FrameProfiler;
use gpu::ResourceFactories;
use crate::render::pass::temporal_denoise::denoise_signal::DenoiseSignal;
use crate::render::pass::temporal_denoise::temporal_denoise_pass::TemporalDenoisePass;
use crate::render::frame_data::culling_view_gpu::CullingViewGPU;
use crate::render::pass::culling_indirect::cull_request::CullRequest;
use render_graph::DrawBucket;
use crate::render::pass::draw_pool::DrawPool;
use crate::render::pass::culling_indirect::culling_indirect_pass::CullingIndirectPass;
use crate::render::pass::culling_indirect::cull_request_statistics::CASCADE_CULLING_META_NAME;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::shadows::cascade_compute::cascade_compute_pass::CascadeComputePass;
use crate::render::pass::shadows::depth_reduce::depth_reduce_pass::DepthReducePass;
use crate::render::pass::shadows::rt_shadow::rt_shadow_pass::RTShadowPass;
use crate::render::pass::shadows::rt_transmissive_shadow::rt_transmissive_shadow_pass::RTTransmissiveShadowPass;
use crate::render::pass::shadows::shadow_resolve::shadow_resolve_pass::ShadowResolvePass;
use crate::render::pass::shadows::cascade_shadows::cascade_shadows_pass::CascadeShadowsPass;
use render_graph::PassGraph;
use render_graph::VirtualAccelerationStructure;
use render_graph::BufferBlueprint;
use render_graph::VirtualBuffer;
use render_graph::ImageBlueprint;
use render_graph::ImageSize;
use render_graph::VirtualImage;
use crate::render::frame_data::shadow_cascades_buffer::ShadowCascadeGPU;
use settings::RenderSettings;

pub struct Shadows {
    pub history: [VirtualImage; 2],
    pub colored: bool,
}

impl Shadows {
    pub fn build(
        pass_graph: &mut PassGraph,
        resources: &PassResources,
        profiler: &FrameProfiler,
        resource_factories: &ResourceFactories,
        settings: &RenderSettings,
        ray_tracing_supported: bool,
        limits: &RenderLimits,
        depth_image: VirtualImage,
        normal_image: VirtualImage,
        velocity_image: VirtualImage,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        bone_transform: VirtualBuffer,
        draw_pool: DrawPool,
        shadow_bucket: DrawBucket,
        cascade_cull_requests_buffer: VirtualBuffer,
        guide_a: VirtualImage,
        guide_b: VirtualImage,
        tlas: Option<VirtualAccelerationStructure>,
        shared_render_settings: VirtualData<RenderSettings>,
        shared_render_snapshot: VirtualData<RenderSnapshot>,
    ) -> Result<Self> {
        let rt_shadows = ray_tracing_supported && settings.rt_shadows.value;
        let shadow_enabled = settings.shadow_enabled.value;
        let transmissive = rt_shadows && settings.transmissive_shadows.value;
        let denoise = settings.shadow_denoise.value;

        let shadow_raw_image = pass_graph.create_image(
            "shadow_raw",
            ImageBlueprint::storage(
                ImageSize::render_full(),
                if transmissive {
                    Format::R16G16B16A16_SFLOAT
                } else {
                    Format::R16_SFLOAT
                },
            ),
        );

        let (true, Some(tlas)) = (rt_shadows, tlas) else {
            let shadow_map_image = pass_graph.create_image(
                "global_shadow_array",
                ImageBlueprint::shadow_map(
                    ImageSize::absolute(
                        limits.shadow_map_limits.resolution,
                        limits.shadow_map_limits.resolution,
                    ),
                    limits.shadow_map_limits.format.vulkan(),
                    limits.shadow_map_limits.cascade_count,
                ),
            );
            let shadow_cascades_buffer = pass_graph.create_buffer(
                "shadow_cascades",
                BufferBlueprint::storage(
                    limits.shadow_map_limits.cascade_count as DeviceSize
                        * size_of::<ShadowCascadeGPU>() as DeviceSize,
                ),
            );

            if shadow_enabled {
                let depth_reduce_result_buffer = pass_graph.import_buffer_placeholder("depth_reduce_result");
                let cascade_culling_views_buffer = pass_graph.create_buffer(
                    "cascade_culling_views",
                    BufferBlueprint::storage(
                        limits.shadow_map_limits.cascade_count as DeviceSize
                            * size_of::<CullingViewGPU>() as DeviceSize,
                    ),
                );

                pass_graph.add_pass(
                    DepthReducePass::create(
                        resources,
                        depth_image,
                        depth_reduce_result_buffer,
                        limits.shadow_map_limits.z_far_sample_stride,
                    )?,
                    profiler,
                );
                pass_graph.add_pass(
                    CascadeComputePass::create(
                        resources,
                        limits.shadow_map_limits,
                        resource_factories,
                        limits.frames_in_flight,
                        scene_buffer,
                        depth_reduce_result_buffer,
                        cascade_culling_views_buffer,
                        shadow_cascades_buffer,
                    )?,
                    profiler,
                );
                pass_graph.add_pass(
                    CullingIndirectPass::create(
                        resources,
                        limits.frames_in_flight,
                        resource_factories,
                        "cascade_culling_indirect",
                        CASCADE_CULLING_META_NAME,
                        limits.shadow_map_limits.cascade_count,
                        true,
                        scene_buffer,
                        entity_buffer,
                        cascade_culling_views_buffer,
                        draw_pool,
                        vec![
                            CullRequest { accept_mask: MaterialGPU::FLAG_ALPHA_OPAQUE | MaterialGPU::FLAG_ALPHA_MASK, bucket: shadow_bucket },
                        ],
                        cascade_cull_requests_buffer,
                        shared_render_snapshot,
                    )?,
                    profiler,
                );
                pass_graph.add_pass(
                    CascadeShadowsPass::create(
                        resources,
                        limits.shadow_map_limits.cascade_count,
                        limits.shadow_map_limits.format.vulkan(),
                        shadow_map_image,
                        entity_buffer,
                        shadow_cascades_buffer,
                        draw_pool,
                        shadow_bucket,
                        bone_transform,
                    )?,
                    profiler,
                );
            }

            pass_graph.add_pass(
                ShadowResolvePass::create(
                    resources,
                    depth_image,
                    normal_image,
                    shadow_map_image,
                    shadow_raw_image,
                    scene_buffer,
                    shadow_cascades_buffer,
                    limits.shadow_map_limits,
                    shared_render_settings,
                )?,
                profiler,
            );

            return Ok(Self {
                history: [shadow_raw_image, shadow_raw_image],
                colored: false,
            });
        };

        if transmissive {
            pass_graph.add_pass(
                RTTransmissiveShadowPass::create(
                    resources,
                    depth_image,
                    normal_image,
                    shadow_raw_image,
                    scene_buffer,
                    entity_buffer,
                    tlas,
                    shared_render_settings,
                )?,
                profiler,
            );
        } else {
            pass_graph.add_pass(
                RTShadowPass::create(
                    resources,
                    depth_image,
                    normal_image,
                    shadow_raw_image,
                    scene_buffer,
                    tlas,
                    shared_render_settings,
                )?,
                profiler,
            );
        }

        if !denoise {
            return Ok(Self {
                history: [shadow_raw_image, shadow_raw_image],
                colored: transmissive,
            });
        }

        let history: [VirtualImage; 2] = from_fn(|index| {
            pass_graph.create_image(
                if index == 0 {
                    "shadow_history_a"
                } else {
                    "shadow_history_b"
                },
                ImageBlueprint::storage(ImageSize::render_full(), Format::R16G16B16A16_SFLOAT),
            )
        });

        pass_graph.add_pass(
            TemporalDenoisePass::create(
                resources,
                shadow_raw_image,
                velocity_image,
                guide_a,
                guide_b,
                history[0],
                history[1],
                DenoiseSignal::Shadow { colored: transmissive },
                shared_render_settings,
            )?,
            profiler,
        );

        Ok(Self { history, colored: transmissive })
    }
}
