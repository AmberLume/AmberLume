use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, BlendFactor, BlendOp, Buffer, ColorComponentFlags, CompareOp, CullModeFlags, DependencyFlags, DeviceAddress, DeviceSize, Format, FrontFace, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::ids::SliceIndex;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::ui::ui_push_constants::UiPushConstants;
use crate::render::pass::ui::ui_snapshot::UiDrawLayer;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::render_targets::{ColorTarget, RenderTargets};
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::resources::store::providers::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::store::providers::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::resource_manifest::shaders;

pub struct UiPass {
    _handle: Arc<ResRef>,
    
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    target_image: VirtualImage,
}

impl UiPass {
    pub fn create(
        color_format: Format,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        target_image: VirtualImage,
    ) -> Result<Self> {
        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: shaders::YAKUI_FRAG,
                fn_name: String::from("main"),
                stage: ShaderStageFlags::FRAGMENT,
            },
            PipelineStageConfig {
                shader_name: shaders::YAKUI_VERT,
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let pipeline_config = PipelineConfig {
            label: "ui".to_string(),
            
            stages: pipeline_stages,

            color_formats: vec![color_format],
            depth_format: None,
            view_mask: 0,

            cull_mode: CullModeFlags::NONE,
            polygon_mode: PolygonMode::FILL,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            primitive_topology: PrimitiveTopology::TRIANGLE_LIST,

            depth_bias_enable: false,
            depth_bias_constant_factor: 0.0,
            depth_bias_slope_factor: 0.0,
            
            depth_test: false,
            depth_write: false,
            depth_compare_op: CompareOp::LESS_OR_EQUAL,

            msaa_samples: SampleCountFlags::TYPE_1,

            blend_enabled: true,
            color_blend: Some(BlendConfig {
                blend_op: BlendOp::ADD,
                src_blend: BlendFactor::ONE,
                dst_blend: BlendFactor::ONE_MINUS_SRC_ALPHA,
            }),
            alpha_blend: Some(BlendConfig {
                blend_op: BlendOp::ADD,
                src_blend: BlendFactor::ONE,
                dst_blend: BlendFactor::ONE_MINUS_SRC_ALPHA,
            }),
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
        })
    }
}

pub struct UiRenderPassData {
    indices_handle: Buffer,
    indices_offset: DeviceSize,

    vertices: DeviceAddress,

    ui_draw_layers: Vec<UiDrawLayer>,
}

impl Pass for UiPass {
    type PassData = UiRenderPassData;

    fn name(&self) -> String {
        String::from("ui")
    }
    
    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(
        &self,
        context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let indices_buffer_view = context.ui_context.index_buffer
            .frame(context.frame_index)
            .slice_at(SliceIndex::ZERO);
        let vertices_buffer_view = context.ui_context.vertex_buffer
            .frame(context.frame_index)
            .slice_at(SliceIndex::ZERO);

        let indices_barrier = indices_buffer_view
            .stage(&context.ui_snapshot.indices, AccessFlags::SHADER_READ)?;
        let vertices_barrier = vertices_buffer_view
            .stage(&context.ui_snapshot.vertices, AccessFlags::SHADER_READ)?;

        context.pipeline_barrier(
            PipelineStageFlags::HOST,
            PipelineStageFlags::VERTEX_SHADER,
            DependencyFlags::empty(),
            &[
                indices_barrier,
                vertices_barrier,
            ],
        );

        Ok(UiRenderPassData {
            indices_handle: indices_buffer_view.handle(),
            indices_offset: indices_buffer_view.offset(),

            vertices: vertices_buffer_view.device_address(),

            ui_draw_layers: context.ui_snapshot.draw_layers.clone(),
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .read_image(
                self.target_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_READ,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .write_image(
                self.target_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            );
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        Some(RenderTargets {
            color: vec![ColorTarget { image: self.target_image, mip: None, clear: None }],
            depth: None,
            view_mask: 0,
        })
    }

    fn record_commands(&self, context: &PassContext, image_scope: &ImageResourceScope, _buffer_scope: &BufferResourceScope, data: Self::PassData) -> Result<()> {
        let target_image = image_scope.get_physical_image(self.target_image);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.bind_ui_index_buffer(data.indices_handle, data.indices_offset);

        data.ui_draw_layers.iter().for_each(|draw_layer| {
            draw_layer.draw_calls.iter().for_each(|draw_call| {
                if let Some(clip_area) = &draw_call.clip {
                    context.set_area_scissor(&clip_area);
                } else {
                    context.set_image_scissor(&target_image);
                }

                context.push_constants(
                    self.pipeline_layout,
                    &UiPushConstants::create(
                        data.vertices,
                        draw_call.texture_index,
                        draw_call.render_mode as u32,
                    ),
                );

                context.draw_indexed(
                    draw_call.index_count,
                    draw_call.index_offset,
                    draw_call.vertex_offset,
                );
            });
        });

        Ok(())
    }
    
    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("MainRenderPass destroyed");

        Ok(())
    }
}
