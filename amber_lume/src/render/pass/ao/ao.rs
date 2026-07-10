use crate::profiler::frame_profiler::FrameProfiler;
use crate::render::pass::ao::gtao::gtao_pass::GtaoPass;
use crate::render::pass::ao::guide::denoise_guide_pass::DenoiseGuidePass;
use crate::render::pass::ao::rt_ao::rt_ao_pass::RTAOPass;
use crate::render::pass::ao::temporal::temporal_pass::{DenoiseSignal, GtaoTemporalPass};
use crate::render::pass::pass_resources::PassResources;
use crate::render::render_graph::pass_graph::PassGraph;
use crate::render::render_graph::virtual_acceleration_structure::virtual_acceleration_structure::VirtualAccelerationStructure;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::image_size::ImageSize;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use anyhow::Result;
use ash::vk::Format;
use std::array::from_fn;

pub struct Ao {
    pub raw: VirtualImage,
    pub history: [VirtualImage; 2],
    pub guide: [VirtualImage; 2],
}

impl Ao {
    pub fn build(
        pass_graph: &mut PassGraph,
        resources: &PassResources,
        profiler: &FrameProfiler,
        depth_image: VirtualImage,
        normal_image: VirtualImage,
        velocity_image: VirtualImage,
        scene_buffer: VirtualBuffer,
        rt_ao: bool,
        tlas: Option<VirtualAccelerationStructure>,
    ) -> Result<Self> {
        let raw = pass_graph.create_image(
            "gtao",
            ImageBlueprint::storage(ImageSize::render_full(), Format::R16G16_SFLOAT),
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
                    "gtao_history_a"
                } else {
                    "gtao_history_b"
                },
                ImageBlueprint::storage(ImageSize::render_full(), Format::R16G16B16A16_SFLOAT),
            )
        });

        if let (true, Some(tlas)) = (rt_ao, tlas) {
            pass_graph.add_pass(
                RTAOPass::create(
                    resources,
                    depth_image,
                    normal_image,
                    raw,
                    tlas,
                )?,
                profiler,
            );
        } else {
            pass_graph.add_pass(
                GtaoPass::create(
                    resources,
                    depth_image,
                    normal_image,
                    raw,
                    scene_buffer,
                )?,
                profiler,
            );
        }
        pass_graph.add_pass(
            DenoiseGuidePass::create(resources, depth_image, normal_image, guide[0], guide[1])?,
            profiler,
        );
        pass_graph.add_pass(
            GtaoTemporalPass::create(
                resources,
                raw,
                velocity_image,
                guide[0],
                guide[1],
                history[0],
                history[1],
                DenoiseSignal::Ao { rt_mode: rt_ao && tlas.is_some() },
            )?,
            profiler,
        );

        Ok(Self { raw, history, guide })
    }
}
