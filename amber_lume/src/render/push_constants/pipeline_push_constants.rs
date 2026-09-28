use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::render::push_constants::push_constants::PushConstants;
use crate::render::push_constants::vertex_shader::VertexShader;
use ash::vk::PipelineLayout;
use bytemuck::bytes_of;
use gpu::PipelineLayoutFactory;
use pipeline_store::PipelineStageConfig;
use render_graph::FrameContext;
use std::iter::once;
use std::marker::PhantomData;
use std::mem::offset_of;

pub struct PipelinePushConstants<V: VertexShader, F: FragmentShader> {
    push_constants: PhantomData<PushConstants<V, F>>,
}

impl<V: VertexShader, F: FragmentShader> PipelinePushConstants<V, F> {
    pub fn new() -> Self {
        const {
            assert!(
                size_of::<PushConstants<V, F>>() <= PipelineLayoutFactory::PUSH_CONSTANTS_SIZE as usize,
                "Vertex and fragment push constants exceed the pipeline layout budget",
            );
        }

        Self {
            push_constants: PhantomData,
        }
    }

    pub fn stages_layout(&self) -> Vec<PipelineStageConfig> {
        once(PipelineStageConfig::vertex(V::SHADER))
            .chain(F::SHADER.map(PipelineStageConfig::fragment))
            .collect()
    }

    pub fn push(&self, context: &FrameContext, pipeline_layout: PipelineLayout, push_constants: &PushConstants<V, F>) {
        let mut bytes = [0u8; PipelineLayoutFactory::PUSH_CONSTANTS_SIZE as usize];

        let vertex_offset = offset_of!(PushConstants<V, F>, vertex);
        let fragment_offset = offset_of!(PushConstants<V, F>, fragment);

        bytes[vertex_offset..vertex_offset + size_of::<V>()].copy_from_slice(bytes_of(&push_constants.vertex));
        bytes[fragment_offset..fragment_offset + size_of::<F>()].copy_from_slice(bytes_of(&push_constants.fragment));

        context.push_constants(pipeline_layout, &bytes);
    }
}
