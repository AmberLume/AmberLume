use render_graph::VirtualData;
use settings::RenderSettings;
use gpu::FrameProfiler;
use crate::render::pass::ao::gtao::gtao_pass::GtaoPass;
use crate::render::pass::ao::ao_spatial::ao_spatial_pass::AoSpatialPass;
use crate::render::pass::ao::gtao_depth::gtao_depth_pass::GtaoDepthPass;
use crate::render::pass::ao::gtao_depth_mip::gtao_depth_mip_pass::GtaoDepthMipPass;
use crate::render::pass::ao::guide::denoise_guide_pass::DenoiseGuidePass;
use crate::render::pass::ao::rt_ao::rt_ao_pass::RTAOPass;
use crate::render::pass::temporal_denoise::denoise_signal::DenoiseSignal;
use crate::render::pass::temporal_denoise::temporal_denoise_pass::TemporalDenoisePass;
use crate::render::pass_resources::pass_resources::PassResources;
use crate::render::pass_resources::ray_tracing_handles::RayTracingHandles;
use render_graph::PassGraph;
use render_graph::VirtualBuffer;
use render_graph::ImageBlueprint;
use render_graph::ImageSize;
use render_graph::VirtualImage;
use anyhow::Result;
use ash::vk::Format;
use gpu::ImageViewDescription;
use std::array::from_fn;

pub struct Ao {
    pub raw: VirtualImage,
    pub history: [VirtualImage; 2],
    pub guide: [VirtualImage; 2],
}

impl Ao {
    pub const VIEW_Z_MIP_COUNT: u32 = 5;

    pub fn build(
        pass_graph: &mut PassGraph,
        resources: &PassResources,
        profiler: &FrameProfiler,
        depth_image: VirtualImage,
        normal_image: VirtualImage,
        velocity_image: VirtualImage,
        camera_buffer: VirtualBuffer,
        ao_spatial: bool,
        ray_tracing_handles: Option<RayTracingHandles>,
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let raw = pass_graph.create_image(
            "ao_raw",
            ImageBlueprint::storage(ImageSize::render_full(), Format::R16_SFLOAT),
        );
        let guide: [VirtualImage; 2] = from_fn(|index| {
            pass_graph.create_image(
                if index == 0 {
                    "denoise_guide_a"
                } else {
                    "denoise_guide_b"
                },
                ImageBlueprint::storage(ImageSize::render_full(), Format::R16G16B16A16_SFLOAT),
            )
        });
        let history: [VirtualImage; 2] = from_fn(|index| {
            pass_graph.create_image(
                if index == 0 {
                    "ao_history_a"
                } else {
                    "ao_history_b"
                },
                ImageBlueprint::storage(ImageSize::render_full(), Format::R16G16B16A16_SFLOAT),
            )
        });

        let traced = if ao_spatial {
            pass_graph.create_image(
                "ao_traced",
                ImageBlueprint::storage(ImageSize::render_full(), Format::R16_SFLOAT),
            )
        } else {
            raw
        };

        if let Some(ray_tracing_handles) = ray_tracing_handles {
            pass_graph.add_pass(
                RTAOPass::create(
                    resources,
                    depth_image,
                    normal_image,
                    traced,
                    camera_buffer,
                    ray_tracing_handles.tlas,
                    ray_tracing_handles.blas,
                    render_settings,
                )?,
                profiler,
            );
        } else {
            let view_z = pass_graph.create_image(
                "gtao_view_z",
                ImageBlueprint {
                    image_view_description: ImageViewDescription {
                        level_count: Self::VIEW_Z_MIP_COUNT,
                        ..ImageViewDescription::default_2d_color()
                    },
                    ..ImageBlueprint::storage(ImageSize::render_full(), Format::R16_SFLOAT)
                },
            );

            pass_graph.add_pass(
                GtaoDepthPass::create(resources, depth_image, view_z, camera_buffer, render_settings)?,
                profiler,
            );

            for level in 1..Self::VIEW_Z_MIP_COUNT {
                pass_graph.add_pass(
                    GtaoDepthMipPass::create(resources, view_z, level - 1, level, render_settings)?,
                    profiler,
                );
            }
            pass_graph.add_pass(
                GtaoPass::create(
                    resources,
                    view_z,
                    normal_image,
                    traced,
                    camera_buffer,
                    render_settings,
                )?,
                profiler,
            );
        }
        pass_graph.add_pass(
            DenoiseGuidePass::create(resources, depth_image, normal_image, guide[0], guide[1], camera_buffer, render_settings)?,
            profiler,
        );
        if ao_spatial {
            pass_graph.add_pass(
                AoSpatialPass::create(resources, traced, guide, raw, camera_buffer, render_settings)?,
                profiler,
            );
        }
        pass_graph.add_pass(
            TemporalDenoisePass::create(
                resources,
                raw,
                velocity_image,
                guide[0],
                guide[1],
                history[0],
                history[1],
                DenoiseSignal::Ao,
                render_settings,
            )?,
            profiler,
        );

        Ok(Self { raw, history, guide })
    }
}
