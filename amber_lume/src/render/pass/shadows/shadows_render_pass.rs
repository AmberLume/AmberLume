use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::pass::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ClearDepthStencilValue, ClearValue, ColorComponentFlags, CompareOp, CullModeFlags, Extent2D, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfo, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::ids::FrameIndex;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::shadows::shadows_push_constants::ShadowsPushConstants;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resources::resource_context::ResourceContext;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::resources::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};

pub struct ShadowsPass {
    _handle: Arc<ResRef>, 
    
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    
    persistent_resources: Arc<PersistentResources>,

    buffer_manager: Arc<BufferManager>,
}

impl ShadowsPass {
    pub fn create(
        resource_context: &ResourceContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        persistent_resources: Arc<PersistentResources>,
    ) -> Result<Self> {
        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: String::from("shaders/shadows/shadows.vert.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let pipeline_config = PipelineConfig {
            label: "shadows".to_string(),
            
            stages: pipeline_stages,

            color_formats: vec![],
            depth_format: Some(persistent_resources.shadows.global_shadow_array.image_description.format),

            cull_mode: CullModeFlags::NONE,
            polygon_mode: PolygonMode::FILL,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            primitive_topology: PrimitiveTopology::TRIANGLE_LIST,

            depth_bias_enable: true,
            depth_bias_constant_factor: 1.5,
            depth_bias_slope_factor: 2.0,

            depth_test: true,
            depth_write: true,
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
            
            buffer_manager: resource_context.buffer_manager.clone(),
            
            persistent_resources,
        })
    }
}

impl Pass for ShadowsPass {
    type PassData = ();
    type Statistics = ();

    fn name(&self) -> String {
        String::from("shadows")
    }
    
    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(&self, _context: &FrameDataContext) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, _context: &PassContext, declaration: &mut PassResourceDeclaration) {
        let global_shadow_image = &self.persistent_resources.shadows.global_shadow_array;
        
        declaration
            .image(
                global_shadow_image.image,
                global_shadow_image.image_subresource_range,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            );
    }
    
    fn record_commands(&self, context: &PassContext, _data: Self::PassData) -> Result<()> {
        let global_shadow_image = &self.persistent_resources.shadows.global_shadow_array;
        
        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        let extent = self.persistent_resources.shadows.global_shadow_array.image_description.extent;
        let extent = Extent2D {
            width: extent.width,
            height: extent.height,
        };
        
        context.set_image_scissor(extent);
        context.set_viewport(extent);

        context.bind_index_buffer();
        
        for shadow_cascade_index in 0..context.render_views_layout.global_shadow_cascades.len() {
            let layer_image_view = global_shadow_image.image_view_layers[shadow_cascade_index];

            let depth_attachment = RenderingAttachmentInfoKHR::default()
                .image_view(layer_image_view)
                .image_layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .load_op(AttachmentLoadOp::CLEAR)
                .store_op(AttachmentStoreOp::STORE)
                .clear_value(ClearValue {
                    depth_stencil: ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                });

            let rendering_info = RenderingInfo::default()
                .render_area(Rect2D {
                    offset: Offset2D { x: 0, y: 0 },
                    extent: Extent2D {
                        width: global_shadow_image.image_description.extent.width,
                        height: global_shadow_image.image_description.extent.height,
                    },
                })
                .layer_count(1)
                .depth_attachment(&depth_attachment);

            context.begin_rendering(&rendering_info);

            let shadow_chunk_index = context.render_views_layout.get_shadow_cascade_index(shadow_cascade_index as u32);
            context.push_constants(
                self.pipeline_layout,
                &ShadowsPushConstants::create(
                    self.buffer_manager.scene_buffer.frame(context.frame_index),
                    self.buffer_manager.draw_data_buffer.chunk(shadow_chunk_index),
                    self.buffer_manager.entity_buffer.frame(context.frame_index),
                    context.resource_buffers.vertex_buffer,
                    shadow_cascade_index as u32,
                ),
            );
            context.draw_indirect_gpu_scene(
                &self.buffer_manager.indirect_buffer.chunk(shadow_chunk_index),
                &self.buffer_manager.draw_count_buffer.chunk(shadow_chunk_index),
            );

            context.end_rendering();
        };
        
        Ok(())
    }

    fn statistics(&self, _frame_index: FrameIndex) -> Self::Statistics {
        ()
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("ShadowsRenderPass destroyed");

        Ok(())
    }
}
