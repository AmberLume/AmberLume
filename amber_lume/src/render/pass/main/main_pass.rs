use crate::render::pass::main::main_push_constants::MainPushConstants;
use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_context::RenderContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, Format, FrontFace, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::virtual_image::render_targets::{ClearColor, ColorTarget, DepthTarget, RenderTargets};
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::resource_manifest::shaders;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::resources::store::providers::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::store::providers::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};

pub struct MainPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    target_image: VirtualImage,
    entity_id_image: VirtualImage,
    depth: VirtualImage,
    shadows: VirtualImage,

    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    shadow_cascades_buffer: VirtualBuffer,
    draw_count_main: VirtualBuffer,
    indirect_main: VirtualBuffer,
    draw_data_main: VirtualBuffer,
    bone_transform: VirtualBuffer,
}

impl MainPass {
    pub fn create(
        color_format: Format,
        render_context: &RenderContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        target_image: VirtualImage,
        entity_id_image: VirtualImage,
        depth: VirtualImage,
        shadows: VirtualImage,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        shadow_cascades_buffer: VirtualBuffer,
        draw_count_main: VirtualBuffer,
        indirect_main: VirtualBuffer,
        draw_data_main: VirtualBuffer,
        bone_transform: VirtualBuffer,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "main".to_string(),

            stages: vec![
                PipelineStageConfig {
                    shader_name: shaders::MAIN_FRAG,
                    fn_name: String::from("main"),
                    stage: ShaderStageFlags::FRAGMENT,
                },
                PipelineStageConfig {
                    shader_name: shaders::MAIN_VERT,
                    fn_name: String::from("main"),
                    stage: ShaderStageFlags::VERTEX,
                },
            ],

            color_formats: vec![color_format, Format::R32_UINT],
            depth_format: Some(render_context.depth_format),
            view_mask: 0,

            cull_mode: CullModeFlags::BACK,
            polygon_mode: PolygonMode::FILL,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            primitive_topology: PrimitiveTopology::TRIANGLE_LIST,

            depth_bias_enable: false,
            depth_bias_constant_factor: 0.0,
            depth_bias_slope_factor: 0.0,

            depth_test: true,
            depth_write: false,
            depth_compare_op: CompareOp::LESS_OR_EQUAL,

            msaa_samples: SampleCountFlags::TYPE_1,

            blend_enabled: false,
            color_blend: Some(BlendConfig {
                blend_op: BlendOp::ADD,
                src_blend: BlendFactor::ONE,
                dst_blend: BlendFactor::ZERO,
            }),
            alpha_blend: None,
            color_write_mask: ColorComponentFlags::RGBA,
        };

        let _handle = pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            target_image,
            entity_id_image,
            depth,
            shadows,

            scene_buffer,
            entity_buffer,
            shadow_cascades_buffer,
            draw_count_main,
            indirect_main,
            draw_data_main,
            bone_transform,
        })
    }
}

impl Pass for MainPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("main")
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(
        &self,
        _context: &FrameDataContext,
        _resource_registry: &mut ResourceRegistry,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .write_image(
                self.depth,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE | AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            )
            .read_image(
                self.shadows,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .write_image(
                self.target_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .write_image(
                self.entity_id_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .read_buffer(
                self.scene_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.entity_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.shadow_cascades_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.draw_count_main,
                AccessFlags::INDIRECT_COMMAND_READ,
                PipelineStageFlags::DRAW_INDIRECT,
            )
            .read_buffer(
                self.indirect_main,
                AccessFlags::INDIRECT_COMMAND_READ,
                PipelineStageFlags::DRAW_INDIRECT,
            )
            .read_buffer(
                self.draw_data_main,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            )
            .read_buffer(
                self.bone_transform,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            );
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        Some(RenderTargets {
            color: vec![
                ColorTarget {
                    image: self.target_image,
                    clear: None,
                },
                ColorTarget {
                    image: self.entity_id_image,
                    clear: Some(ClearColor::Uint([u32::MAX; 4])),
                },
            ],
            depth: Some(DepthTarget {
                image: self.depth,
                clear: None,
            }),
            view_mask: 0,
        })
    }

    fn record_commands(&self, context: &PassContext, resource_registry: &ResourceRegistry, _data: Self::PassData) -> Result<()> {
        let shadows_image = resource_registry.get_physical_image(self.shadows);

        let scene_buffer = resource_registry.get_physical_buffer(self.scene_buffer);
        let entity_buffer = resource_registry.get_physical_buffer(self.entity_buffer);
        let shadow_cascades_buffer = resource_registry.get_physical_buffer(self.shadow_cascades_buffer);
        let draw_count_main = resource_registry.get_physical_buffer(self.draw_count_main);
        let indirect_main = resource_registry.get_physical_buffer(self.indirect_main);
        let draw_data_main_buffer = resource_registry.get_physical_buffer(self.draw_data_main);
        let bone_transform_buffer = resource_registry.get_physical_buffer(self.bone_transform);

        context.bind_index_buffer();

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);
        context.push_constants(
            self.pipeline_layout,
            &MainPushConstants::create(
                scene_buffer,
                draw_data_main_buffer,
                context.resource_buffers.vertex_buffer,
                entity_buffer,
                context.resource_buffers.submesh_buffer,
                context.resource_buffers.material_buffer,
                bone_transform_buffer,
                shadows_image.descriptors.full.unwrap(),
                shadow_cascades_buffer,
                context.limits.shadow_map_limits.bias,
                context.limits.shadow_map_limits.normal_bias,
                context.limits.shadow_map_limits.pcf_world_radius,
                context.limits.shadow_map_limits.pcf_sample_count,
                context.limits.shadow_map_limits.cascade_blend_range,
            ),
        );
        context.draw_indirect_gpu_scene(
            &indirect_main,
            &draw_count_main,
        );

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("MainRenderPass destroyed");

        Ok(())
    }
}
