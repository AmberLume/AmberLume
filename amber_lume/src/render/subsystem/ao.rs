use anyhow::Result;
use ash::vk::Format;
use std::array::from_fn;

use crate::profiler::frame_profiler::FrameProfiler;
use crate::render::pass::gtao::gtao_pass::GtaoPass;
use crate::render::pass::gtao::temporal_pass::GtaoTemporalPass;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::rt_ao::rt_ao_pass::RTAOPass;
use crate::render::render_graph::pass_graph::PassGraph;
use crate::render::render_graph::virtual_acceleration_structure::virtual_acceleration_structure::VirtualAccelerationStructure;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::image_size::ImageSize;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;

pub struct Ao {
    pub raw: VirtualImage,
    pub history: [VirtualImage; 2],
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
            ImageBlueprint::storage(ImageSize::Render { pow: 1 }, Format::R16_SFLOAT),
        );
        let history: [VirtualImage; 2] = from_fn(|index| {
            pass_graph.create_image(
                if index == 0 { "gtao_history_a" } else { "gtao_history_b" },
                ImageBlueprint::storage(ImageSize::Render { pow: 1 }, Format::R16_SFLOAT),
            )
        });

        if let (true, Some(tlas)) = (rt_ao, tlas) {
            pass_graph.add_pass(
                RTAOPass::create(resources, depth_image, normal_image, raw, tlas)?,
                profiler,
            );
        } else {
            pass_graph.add_pass(
                GtaoPass::create(resources, depth_image, normal_image, raw, scene_buffer)?,
                profiler,
            );
        }
        pass_graph.add_pass(
            GtaoTemporalPass::create(resources, raw, velocity_image, history[0], history[1])?,
            profiler,
        );

        Ok(Self { raw, history })
    }
}
