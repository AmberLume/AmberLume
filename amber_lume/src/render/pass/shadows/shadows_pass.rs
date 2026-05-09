use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ClearDepthStencilValue, ClearValue, ColorComponentFlags, CompareOp, CullModeFlags, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfo, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::ids::{FrameIndex, SliceIndex};
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::shadows::shadows_push_constants::ShadowsPushConstants;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::resources::resource_context::ResourceContext;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::resources::persistent_shadows::PersistentShadows;
use crate::resources::store::providers::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::store::providers::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;

pub struct ShadowsPass {
    _handle: Arc<ResRef>,
    
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    buffer_manager: Arc<BufferManager>,

    shadows_image: VirtualImage,

    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    shadow_cascades_buffer: VirtualBuffer,
}

impl ShadowsPass {
    pub fn create(
        resource_context: &ResourceContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        persistent_shadows: &PersistentShadows,
        shadows_image: VirtualImage,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        shadow_cascades_buffer: VirtualBuffer,
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
            depth_format: Some(persistent_shadows.global_shadow_array.image_description.format),

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

            shadows_image,

            scene_buffer,
            entity_buffer,
            shadow_cascades_buffer,
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
                self.shadows_image,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
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
                PipelineStageFlags::VERTEX_SHADER,
            );
    }
    
    fn record_commands(&self, context: &PassContext, resource_registry: &ResourceRegistry, _data: Self::PassData) -> Result<()> {
        let shadows_image = resource_registry.get_physical_image(self.shadows_image);

        let scene_buffer = resource_registry.get_physical_buffer(self.scene_buffer);
        let entity_buffer = resource_registry.get_physical_buffer(self.entity_buffer);
        let shadow_cascades_buffer = resource_registry.get_physical_buffer(self.shadow_cascades_buffer);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);
        
        context.set_image_scissor(&shadows_image);
        context.set_viewport(&shadows_image);

        context.bind_index_buffer();
        
        for shadow_cascade_index in 0..context.render_views_layout.cascade_count as usize {
            let layer_image_view = shadows_image.layers[shadow_cascade_index];

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
                    extent: shadows_image.extent,
                })
                .layer_count(1)
                .depth_attachment(&depth_attachment);

            context.begin_rendering(&rendering_info);

            let shadow_chunk_index = context.render_views_layout.get_shadow_cascade_index(shadow_cascade_index as u32);
            context.push_constants(
                self.pipeline_layout,
                &ShadowsPushConstants::create(
                    &scene_buffer,
                    self.buffer_manager.draw_data_buffer.chunk(shadow_chunk_index),
                    &entity_buffer,
                    context.resource_buffers.vertex_buffer,
                    context.bone_transform_handler.bone_transform_buffer.slice_at(SliceIndex::ZERO).device_address(),
                    &shadow_cascades_buffer,
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
