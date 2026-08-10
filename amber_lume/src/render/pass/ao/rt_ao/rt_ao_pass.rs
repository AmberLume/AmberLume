use gpu::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::ao::rt_ao::rt_ao_push_constants::RTAOPushConstants;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::virtual_acceleration_structure::virtual_acceleration_structure::VirtualAccelerationStructure;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use gpu::PipelineLayoutType;
use crate::resource_manifest::shaders;
use pipeline_store::ComputePipelineConfig;
use resource_residency::ResRef;
use settings::AO_TRACE_PERIODS;
use anyhow::{bail, Result};
use ash::vk::{
    AccessFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags,
};
use std::sync::Arc;

pub struct RTAOPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    depth_image: VirtualImage,
    normal_image: VirtualImage,
    ao_image: VirtualImage,
    tlas: VirtualAccelerationStructure,
}

impl RTAOPass {
    pub fn create(
        resources: &PassResources,
        depth_image: VirtualImage,
        normal_image: VirtualImage,
        ao_image: VirtualImage,
        tlas: VirtualAccelerationStructure,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::RT_AO_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources
            .compute_pipeline_provider
            .acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for RTAO");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources
                .pipeline_layout_registry
                .get(PipelineLayoutType::General),
            
            depth_image,
            normal_image,
            ao_image,
            tlas,
        })
    }
}

pub struct RTAOPassData {
    ao_radius: f32,
    sample_count: u32,
    ao_power: f32,
    trace_period: u32,
}

impl Pass for RTAOPass {
    type PassData = RTAOPassData;

    fn name(&self) -> String {
        String::from("rt_ao")
    }

    fn is_enabled(&self, context: &FrameDataContext) -> bool {
        context.render_settings.ao_enabled.value
    }

    fn prepare_data(
        &self,
        context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let settings = context.render_settings;

        Ok(RTAOPassData {
            ao_radius: settings.gtao_radius.value,
            sample_count: settings.ao_samples.value.round().max(1.0) as u32,
            ao_power: settings.gtao_power.value,
            trace_period: AO_TRACE_PERIODS[settings.ao_trace_period.value.min(2)],
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .read_image(
                self.depth_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_image(
                self.normal_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_image(
                self.ao_image,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_acceleration_structure(
                self.tlas,
                AccessFlags::ACCELERATION_STRUCTURE_READ_KHR,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn record_commands(
        &self,
        context: &PassContext,
        image_scope: &ImageResourceScope,
        _buffer_scope: &BufferResourceScope,
        data: Self::PassData,
    ) -> Result<()> {
        let depth_image = image_scope.get_physical_image(self.depth_image);
        let normal_image = image_scope.get_physical_image(self.normal_image);
        let ao_image = image_scope.get_physical_image(self.ao_image);

        let depth_descriptor_id = depth_image
            .descriptors
            .full
            .expect("RTAO depth image must have a sampled descriptor");

        let normal_descriptor_id = normal_image
            .descriptors
            .full
            .expect("RTAO normal image must have a sampled descriptor");

        let ao_storage_id = ao_image
            .descriptors
            .storage_mips
            .as_ref()
            .and_then(|mips| mips.first().copied())
            .expect("RTAO ao image must have a storage descriptor");

        let width = ao_image.extent.width;
        let height = ao_image.extent.height;

        let tlas_descriptor_id = context.frame_index.value;

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);
        context.push_constants(
            self.pipeline_layout,
            &RTAOPushConstants::create(
                &context.render_views_layout.main.jittered_view_projection,
                depth_descriptor_id.inner,
                normal_descriptor_id.inner,
                ao_storage_id.inner,
                width,
                height,
                tlas_descriptor_id,
                data.ao_radius,
                data.sample_count,
                data.ao_power,
                context.frame_number,
                data.trace_period,
            ),
        );

        context.dispatch_2d(width, height);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        Ok(())
    }
}
