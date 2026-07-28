use anyhow::Result;
use ash::vk::{DeviceSize, Format};
use std::array::from_fn;
use crate::limits::AmberLumeLimits;
use crate::profiler::frame_profiler::FrameProfiler;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::temporal_denoise::denoise_signal::DenoiseSignal;
use crate::render::pass::temporal_denoise::temporal_denoise_pass::TemporalDenoisePass;
use crate::render::frame_data::culling_view_gpu::CullingViewGPU;
use crate::render::pass::culling_indirect::cull_request::CullRequest;
use crate::render::pass::draw_bucket::DrawBucket;
use crate::render::pass::draw_pool::DrawPool;
use crate::render::pass::culling_indirect::culling_indirect_pass::CullingIndirectPass;
use crate::render::pass::culling_indirect::cull_request_statistics::CASCADE_CULLING_META_NAME;
use crate::resources::store::providers::material::buffer::materials_buffer::MaterialGPU;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::shadows::cascade_compute::cascade_compute_pass::CascadeComputePass;
use crate::render::pass::shadows::depth_reduce::depth_reduce_pass::DepthReducePass;
use crate::render::pass::shadows::rt_shadow::rt_shadow_pass::RTShadowPass;
use crate::render::pass::shadows::shadow_resolve::shadow_resolve_pass::ShadowResolvePass;
use crate::render::pass::shadows::translucent_shadows::translucent_shadows_pass::TranslucentShadowsPass;
use crate::render::pass::shadows::cascade_shadows::cascade_shadows_pass::CascadeShadowsPass;
use crate::render::render_graph::pass_graph::PassGraph;
use crate::render::render_graph::virtual_acceleration_structure::virtual_acceleration_structure::VirtualAccelerationStructure;
use crate::render::render_graph::virtual_buffer::buffer_blueprint::BufferBlueprint;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::image_size::ImageSize;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::shadow_cascades_buffer::ShadowCascadeGPU;
use crate::settings::settings::EngineSettings;

pub struct Shadows {
    pub history: [VirtualImage; 2],
}

impl Shadows {
    pub fn build(
        pass_graph: &mut PassGraph,
        resources: &PassResources,
        profiler: &FrameProfiler,
        resource_factories: &ResourceFactories,
        settings: &EngineSettings,
        ray_tracing_supported: bool,
        limits: &AmberLumeLimits,
        depth_image: VirtualImage,
        normal_image: VirtualImage,
        velocity_image: VirtualImage,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        bone_transform: VirtualBuffer,
        draw_pool: DrawPool,
        shadow_bucket: DrawBucket,
        shadow_blend_bucket: DrawBucket,
        cascade_cull_requests_buffer: VirtualBuffer,
        guide_a: VirtualImage,
        guide_b: VirtualImage,
        tlas: Option<VirtualAccelerationStructure>,
    ) -> Result<Self> {
        let rt_shadows = ray_tracing_supported && settings.render.rt_shadows.value;
        let shadow_enabled = settings.render.shadow_enabled.value;
        let denoise = settings.render.shadow_denoise.value;

        let shadow_raw_image = pass_graph.create_image(
            "shadow_raw",
            ImageBlueprint::storage(ImageSize::render_full(), Format::R16G16B16A16_SFLOAT),
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
            let translucent_size = ImageSize::absolute(
                limits.shadow_map_limits.translucent_resolution,
                limits.shadow_map_limits.translucent_resolution,
            );
            let shadow_transmittance_image = pass_graph.create_image(
                "shadow_transmittance_array",
                ImageBlueprint::color_array(
                    translucent_size,
                    TranslucentShadowsPass::TRANSMITTANCE_FORMAT,
                    limits.shadow_map_limits.cascade_count,
                ),
            );
            let shadow_translucent_depth_image = pass_graph.create_image(
                "shadow_translucent_depth_array",
                ImageBlueprint::shadow_map(
                    translucent_size,
                    limits.shadow_map_limits.format.vulkan(),
                    limits.shadow_map_limits.cascade_count,
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
                            CullRequest { accept_mask: MaterialGPU::FLAG_ALPHA_BLEND, bucket: shadow_blend_bucket },
                        ],
                        cascade_cull_requests_buffer,
                    )?,
                    profiler,
                );
                pass_graph.add_pass(
                    CascadeShadowsPass::create(
                        resources,
                        "cascade_shadows",
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
                pass_graph.add_pass(
                    CascadeShadowsPass::create(
                        resources,
                        "translucent_depth",
                        limits.shadow_map_limits.cascade_count,
                        limits.shadow_map_limits.format.vulkan(),
                        shadow_translucent_depth_image,
                        entity_buffer,
                        shadow_cascades_buffer,
                        draw_pool,
                        shadow_blend_bucket,
                        bone_transform,
                    )?,
                    profiler,
                );
                pass_graph.add_pass(
                    TranslucentShadowsPass::create(
                        resources,
                        limits.shadow_map_limits.cascade_count,
                        shadow_transmittance_image,
                        entity_buffer,
                        shadow_cascades_buffer,
                        draw_pool,
                        shadow_blend_bucket,
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
                    shadow_transmittance_image,
                    shadow_translucent_depth_image,
                    shadow_raw_image,
                    scene_buffer,
                    shadow_cascades_buffer,
                )?,
                profiler,
            );

            return Ok(Self {
                history: [shadow_raw_image, shadow_raw_image],
            });
        };

        pass_graph.add_pass(
            RTShadowPass::create(
                resources,
                depth_image,
                normal_image,
                shadow_raw_image,
                tlas,
            )?,
            profiler,
        );

        if !denoise {
            return Ok(Self {
                history: [shadow_raw_image, shadow_raw_image],
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
                DenoiseSignal::Shadow,
            )?,
            profiler,
        );

        Ok(Self { history })
    }
}
