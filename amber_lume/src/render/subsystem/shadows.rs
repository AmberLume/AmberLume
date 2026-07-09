use anyhow::Result;
use ash::vk::Format;

use crate::limits::AmberLumeLimits;
use crate::profiler::frame_profiler::FrameProfiler;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::culling_indirect::cascade_culling_indirect_pass::CascadeCullingIndirectPass;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::sdsm::cascade_compute_pass::CascadeComputePass;
use crate::render::pass::sdsm::sdsm_pass::SdsmPass;
use crate::render::pass::rt_shadow::rt_shadow_pass::RTShadowPass;
use crate::render::pass::shadow_resolve::shadow_resolve_pass::ShadowResolvePass;
use crate::render::pass::shadows::shadows_pass::ShadowsPass;
use crate::render::render_graph::pass_graph::PassGraph;
use crate::render::render_graph::virtual_acceleration_structure::virtual_acceleration_structure::VirtualAccelerationStructure;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::image_size::ImageSize;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;

pub struct Shadows {
    shadow_map_image: VirtualImage,
    shadow_cascades_buffer: VirtualBuffer,
    shadow_factor_image: VirtualImage,
    sdsm_result_buffer: VirtualBuffer,
}

impl Shadows {
    pub fn new(
        pass_graph: &mut PassGraph,
        shadow_map_image: VirtualImage,
        shadow_cascades_buffer: VirtualBuffer,
    ) -> Self {
        let shadow_factor_image = pass_graph.create_image(
            "shadow_factor",
            ImageBlueprint::storage(ImageSize::render_full(), Format::R16_SFLOAT),
        );
        let sdsm_result_buffer = pass_graph.import_buffer_placeholder("sdsm_result");

        Self {
            shadow_map_image,
            shadow_cascades_buffer,
            shadow_factor_image,
            sdsm_result_buffer,
        }
    }

    pub fn render_map(
        &self,
        pass_graph: &mut PassGraph,
        resources: &PassResources,
        profiler: &FrameProfiler,
        resource_factories: &ResourceFactories,
        limits: &AmberLumeLimits,
        depth_image: VirtualImage,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        render_view_buffer: VirtualBuffer,
        bone_transform: VirtualBuffer,
        draw_count_shadow: VirtualBuffer,
        indirect_shadow: VirtualBuffer,
        draw_data_shadow: VirtualBuffer,
    ) -> Result<()> {
        pass_graph.add_pass(
            SdsmPass::create(
                resources,
                depth_image,
                self.sdsm_result_buffer,
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
                self.sdsm_result_buffer,
                render_view_buffer,
                self.shadow_cascades_buffer,
            )?,
            profiler,
        );
        pass_graph.add_pass(
            CascadeCullingIndirectPass::create(
                resources,
                &limits.resource_limits,
                limits.frames_in_flight,
                resource_factories,
                scene_buffer,
                entity_buffer,
                render_view_buffer,
                draw_count_shadow,
                indirect_shadow,
                draw_data_shadow,
            )?,
            profiler,
        );
        pass_graph.add_pass(
            ShadowsPass::create(
                resources,
                limits.shadow_map_limits.cascade_count,
                limits.shadow_map_limits.format.vulkan(),
                self.shadow_map_image,
                entity_buffer,
                self.shadow_cascades_buffer,
                draw_count_shadow,
                indirect_shadow,
                draw_data_shadow,
                bone_transform,
            )?,
            profiler,
        );

        Ok(())
    }

    pub fn resolve(
        &self,
        pass_graph: &mut PassGraph,
        resources: &PassResources,
        profiler: &FrameProfiler,
        depth_image: VirtualImage,
        normal_image: VirtualImage,
        scene_buffer: VirtualBuffer,
        rt_shadows: bool,
        tlas: Option<VirtualAccelerationStructure>,
    ) -> Result<VirtualImage> {
        if let (true, Some(tlas)) = (rt_shadows, tlas) {
            pass_graph.add_pass(
                RTShadowPass::create(
                    resources,
                    depth_image,
                    normal_image,
                    self.shadow_factor_image,
                    tlas,
                )?,
                profiler,
            );
        } else {
            pass_graph.add_pass(
                ShadowResolvePass::create(
                    resources,
                    depth_image,
                    normal_image,
                    self.shadow_map_image,
                    self.shadow_factor_image,
                    scene_buffer,
                    self.shadow_cascades_buffer,
                )?,
                profiler,
            );
        }

        Ok(self.shadow_factor_image)
    }
}
