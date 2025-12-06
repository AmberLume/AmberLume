use anyhow::Result;
use ash::vk::{
    ColorComponentFlags, CompareOp, CullModeFlags, Format, FrontFace, PipelineLayout, PolygonMode,
    RenderPass, SampleCountFlags, ShaderModule,
};
use ash::{Device, vk};
use std::array::from_ref;
use std::ffi::CString;
use vk::{
    DynamicState, GraphicsPipelineCreateInfo, Pipeline, PipelineCache,
    PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineDepthStencilStateCreateInfo, PipelineDynamicStateCreateInfo,
    PipelineInputAssemblyStateCreateInfo, PipelineLayoutCreateInfo,
    PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
    PipelineRenderingCreateInfo, PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo,
    PipelineViewportStateCreateInfo, PrimitiveTopology, ShaderStageFlags,
};

pub struct GraphicsPipeline {
    pub pipeline: Pipeline,

    layout: PipelineLayout,
}

impl GraphicsPipeline {
    pub fn create(
        device: &Device,
        vertex_shader_module: ShaderModule,
        fragment_shader_module: ShaderModule,
        color_format: Format,
        depth_format: Format,
    ) -> Result<Self> {
        let main_function_name = CString::new("main")?;

        let shader_stages = [
            PipelineShaderStageCreateInfo::default()
                .stage(ShaderStageFlags::VERTEX)
                .module(vertex_shader_module)
                .name(&main_function_name),
            PipelineShaderStageCreateInfo::default()
                .stage(ShaderStageFlags::FRAGMENT)
                .module(fragment_shader_module)
                .name(&main_function_name),
        ];

        let vertex_input_info = PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&[])
            .vertex_binding_descriptions(&[]);

        let input_assembly_info = PipelineInputAssemblyStateCreateInfo::default()
            .topology(PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport_state = PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_info = PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(PolygonMode::FILL)
            .cull_mode(CullModeFlags::BACK)
            .front_face(FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1.0);

        let multisample_info = PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        let depth_stencil_info = PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        let color_blend_attachment = PipelineColorBlendAttachmentState::default()
            .blend_enable(false)
            .color_write_mask(ColorComponentFlags::RGBA);

        let color_blend_info = PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(from_ref(&color_blend_attachment));

        let dynamic_states = [DynamicState::VIEWPORT, DynamicState::SCISSOR];
        let dynamic_state_info =
            PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let mut rendering_info = PipelineRenderingCreateInfo::default()
            .color_attachment_formats(from_ref(&color_format))
            .depth_attachment_format(depth_format)
            .stencil_attachment_format(Format::UNDEFINED);

        // let layout = GraphicsPipelineLayout::create(&device)?;
        let layout = Self::create_empty_pipeline_layout(&device)?;

        let pipeline_info = GraphicsPipelineCreateInfo::default()
            .push_next(&mut rendering_info)
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_info)
            .depth_stencil_state(&depth_stencil_info)
            .color_blend_state(&color_blend_info)
            .dynamic_state(&dynamic_state_info)
            .layout(layout)
            .render_pass(RenderPass::null())
            .subpass(0);

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(PipelineCache::null(), &[pipeline_info], None)
                .map(|pipelines| pipelines[0])
                .unwrap()
        };

        Ok(Self { pipeline, layout })
    }

    pub fn create_empty_pipeline_layout(device: &Device) -> Result<PipelineLayout> {
        let layout_info = PipelineLayoutCreateInfo::default()
            .set_layouts(&[])
            .push_constant_ranges(&[]);

        let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None)? };

        Ok(pipeline_layout)
    }
}
