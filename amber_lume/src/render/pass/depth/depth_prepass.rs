use std::sync::Arc;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, FrontFace, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, SampleCountFlags, ShaderStageFlags};
use tracing::info;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::depth::depth_push_constants::DepthPushConstants;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_context::RenderContext;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::render_targets::{DepthTarget, RenderTargets};
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::resources::resource_manifest::shaders;
use crate::resources::store::providers::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::store::providers::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;

pub struct DepthPrepass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    depth: VirtualImage,

    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    draw_count_main: VirtualBuffer,
    indirect_main: VirtualBuffer,
    draw_data_main: VirtualBuffer,
    bone_transform: VirtualBuffer,
}

impl DepthPrepass {
    pub fn create(
        render_context: &RenderContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        depth: VirtualImage,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        draw_count_main: VirtualBuffer,
        indirect_main: VirtualBuffer,
        draw_data_main: VirtualBuffer,
        bone_transform: VirtualBuffer,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "depth_prepass".to_string(),

            stages: vec![
                PipelineStageConfig {
                    shader_name: shaders::DEPTH_FRAG,
                    fn_name: String::from("main"),
                    stage: ShaderStageFlags::FRAGMENT,
                },
                PipelineStageConfig {
                    shader_name: shaders::DEPTH_VERT,
                    fn_name: String::from("main"),
                    stage: ShaderStageFlags::VERTEX,
                },
            ],

            color_formats: vec![],
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
            depth_write: true,
            depth_compare_op: CompareOp::LESS,

            msaa_samples: SampleCountFlags::TYPE_1,

            blend_enabled: false,
            color_blend: Some(BlendConfig {
                blend_op: BlendOp::ADD,
                src_blend: BlendFactor::ONE,
                dst_blend: BlendFactor::ZERO,
            }),
            alpha_blend: None,
            color_write_mask: ColorComponentFlags::empty(),
        };

        let _handle = pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline for depth_prepass");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            depth,

            scene_buffer,
            entity_buffer,
            draw_count_main,
            indirect_main,
            draw_data_main,
            bone_transform,
        })
    }
}

impl Pass for DepthPrepass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("depth_prepass")
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(
        &self,
        _context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
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
            .read_buffer(
                self.scene_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            )
            .read_buffer(
                self.entity_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
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
            color: vec![],
            depth: Some(DepthTarget {
                image: self.depth,
                clear: Some(1.0),
            }),
            view_mask: 0,
        })
    }

    fn record_commands(&self, context: &PassContext, _image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope, _data: Self::PassData) -> Result<()> {
        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);
        let entity_buffer = buffer_scope.get_physical_buffer(self.entity_buffer);
        let draw_count_main = buffer_scope.get_physical_buffer(self.draw_count_main);
        let indirect_main = buffer_scope.get_physical_buffer(self.indirect_main);
        let draw_data_main_buffer = buffer_scope.get_physical_buffer(self.draw_data_main);
        let bone_transform_buffer = buffer_scope.get_physical_buffer(self.bone_transform);

        context.bind_index_buffer();

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);
        context.push_constants(
            self.pipeline_layout,
            &DepthPushConstants::create(
                scene_buffer,
                draw_data_main_buffer,
                entity_buffer,
                context.resource_buffers.vertex_buffer,
                bone_transform_buffer,
            ),
        );
        context.draw_indirect_gpu_scene(
            &indirect_main,
            &draw_count_main,
        );

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("DepthPrepass destroyed");

        Ok(())
    }
}
