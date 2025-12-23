use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::buffer::index_buffer::IndexBuffer;
use crate::render::vulkan::render_pass::depth::depth_push_constants::DepthPushConstants;
use crate::render::vulkan::render_pass::render_pass::RenderPass;
use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use crate::render::vulkan::render_pass::utils::transition_image_layout;
use crate::render::vulkan::renderer::render_context::RenderContext;
use crate::resources::model::model_backend::PrimitiveAllocation;
use crate::resources::model::model_config::ModelConfig;
use crate::resources::pipeline::pipeline_config::{PipelineConfig, PipelineStageConfig};
use crate::resources::pipeline_layout::pipeline_layout_config::{
    PipelineLayoutConfig, PushConstantRange,
};
use crate::resources::resource_hub::ResourceHub;
use anyhow::Result;
use ash::vk::{
    AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, ClearDepthStencilValue,
    ClearValue, CompareOp, CullModeFlags, DeviceAddress, Extent2D, FrontFace, ImageLayout,
    IndexType, Offset2D, Pipeline, PipelineLayout, PipelineStageFlags, PolygonMode, Rect2D,
    RenderingAttachmentInfoKHR, RenderingInfoKHR, SampleCountFlags, ShaderStageFlags,
};
use glam::{Mat4, Vec3};
use std::slice::from_raw_parts;
use std::sync::{Arc, Mutex};

pub struct DepthRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    index_buffer: Arc<Mutex<IndexBuffer>>,
    vertex_buffer_device_address: DeviceAddress,

    temp_data_to_draw: Arc<Vec<PrimitiveAllocation>>,
}

impl DepthRenderPass {
    pub fn create(
        render_context: &RenderContext,
        resource_hub: Arc<ResourceHub>,
        buffer_manager: &BufferManager,
    ) -> Result<Self> {
        let format = render_context.render_targets.depth_vulkan_image.format;

        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: String::from("depth.frag.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::FRAGMENT,
            },
            PipelineStageConfig {
                shader_name: String::from("depth.vert.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let pipeline_layout_config = PipelineLayoutConfig {
            descriptor_set_layout_configs: vec![],
            push_constant_ranges: vec![PushConstantRange {
                stage: ShaderStageFlags::VERTEX,
                offset: 0,
                size: size_of::<DepthPushConstants>() as u32,
            }],
        };

        let pipeline_layout = *resource_hub
            .get_pipeline_layout_provider()
            .get_now(&pipeline_layout_config)
            .unwrap();

        let pipeline_config = PipelineConfig {
            stages: pipeline_stages,

            color_formats: vec![],
            depth_format: Some(format),

            cull_mode: CullModeFlags::BACK,
            polygon_mode: PolygonMode::FILL,
            front_face: FrontFace::COUNTER_CLOCKWISE,

            depth_test: true,
            depth_write: true,
            depth_compare_op: CompareOp::LESS,

            msaa_samples: SampleCountFlags::TYPE_1,

            blend: false,
            src_color_blend: BlendFactor::ONE,
            dst_color_blend: BlendFactor::ZERO,

            pipeline_layout_config,
        };

        let pipeline = *resource_hub
            .get_pipeline_provider()
            .get_now(&pipeline_config)
            .unwrap();

        let model_config = ModelConfig {
            name: String::from("1.model"),
        };

        let temp_data_to_draw = resource_hub
            .get_model_provider()
            .get_now(&model_config)
            .unwrap();

        Ok(Self {
            pipeline,
            pipeline_layout,

            index_buffer: buffer_manager.index_buffer.clone(),
            vertex_buffer_device_address: buffer_manager.vertex_buffer_device_address,

            temp_data_to_draw,
        })
    }

    pub fn create_perspective_view_projection(
        field_of_view: f32,
        extent: &Extent2D,
        distance: f32,
        center: Vec3,
    ) -> Mat4 {
        let aspect_ratio = extent.width as f32 / extent.height as f32;

        let camera_position = center + Vec3::new(0.0, distance * 0.707, distance * 0.707);
        let view = Mat4::look_at_rh(camera_position, center, Vec3::Y);

        let field_of_view_radians = field_of_view.to_radians();
        let near = 0.1;
        let far = 1.0;

        let projection = Mat4::perspective_rh(field_of_view_radians, aspect_ratio, near, far);

        projection * view
    }
}

impl RenderPass for DepthRenderPass {
    fn is_enabled(&self) -> bool {
        true
    }

    fn begin_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let depth_image = &render_pass_context
            .render_context
            .render_targets
            .depth_vulkan_image;

        transition_image_layout(
            &render_pass_context,
            depth_image,
            ImageLayout::UNDEFINED,
            ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            AccessFlags::empty(),
            AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE,
            PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
        );

        let depth_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(depth_image.image_view)
            .image_layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .clear_value(ClearValue {
                depth_stencil: ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            });

        let rendering_info = RenderingInfoKHR::default()
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: depth_image.extent,
            })
            .layer_count(1)
            .depth_attachment(&depth_attachment);

        render_pass_context.begin_rendering(&rendering_info);

        Ok(())
    }

    fn record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let command_buffer = render_pass_context.command_recording.command_buffer;
        let device = &render_pass_context.device_context.device;

        render_pass_context.bind_pipeline(self.pipeline);

        render_pass_context.set_scissor();
        render_pass_context.set_viewport();

        let extent = render_pass_context
            .render_context
            .render_targets
            .depth_vulkan_image
            .extent;

        let view_projection = Self::create_perspective_view_projection(
            1.0,
            &extent,
            1.0,
            Vec3::new(0.035, 1.641, 0.121),
        );

        let push_constants = DepthPushConstants {
            view_projection: view_projection.to_cols_array_2d(),
            vertex_buffer_address: self.vertex_buffer_device_address,
        };

        unsafe {
            device.cmd_push_constants(
                command_buffer,
                self.pipeline_layout,
                ShaderStageFlags::VERTEX,
                0,
                from_raw_parts(
                    &push_constants as *const _ as *const u8,
                    size_of::<DepthPushConstants>(),
                ),
            );
        }

        {
            let index_buffer = self.index_buffer.lock().unwrap();

            unsafe {
                device.cmd_bind_index_buffer(
                    command_buffer,
                    index_buffer.buffer.handle,
                    0,
                    IndexType::UINT32,
                );
            }
        }

        for primitive in self.temp_data_to_draw.iter() {
            unsafe {
                device.cmd_draw_indexed(
                    command_buffer,
                    primitive.index_count,
                    1,
                    primitive.index_offset,
                    0,
                    0,
                )
            }
        }

        Ok(())
    }

    fn end_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        render_pass_context.end_rendering();

        Ok(())
    }
}
