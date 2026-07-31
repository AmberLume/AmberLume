use gpu::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::shadows::rt_transmissive_shadow::rt_transmissive_shadow_push_constants::RTTransmissiveShadowPushConstants;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::virtual_acceleration_structure::virtual_acceleration_structure::VirtualAccelerationStructure;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use gpu::PipelineLayoutType;
use crate::resources::resource_manifest::shaders;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::store::providers::res_ref::ResRef;
use anyhow::{bail, Result};
use ash::vk::{
    AccessFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags,
};
use std::sync::Arc;
use index_allocator::ResourceId;

pub struct RTTransmissiveShadowPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    depth_image: VirtualImage,
    normal_image: VirtualImage,
    transmittance_image: VirtualImage,
    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    tlas: VirtualAccelerationStructure,
}

impl RTTransmissiveShadowPass {
    pub fn create(
        resources: &PassResources,
        depth_image: VirtualImage,
        normal_image: VirtualImage,
        transmittance_image: VirtualImage,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        tlas: VirtualAccelerationStructure,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::RT_TRANSMISSIVE_SHADOW_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources
            .compute_pipeline_provider
            .acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for RTTransmissiveShadow");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources
                .pipeline_layout_registry
                .get(PipelineLayoutType::General),

            depth_image,
            normal_image,
            transmittance_image,
            scene_buffer,
            entity_buffer,
            tlas,
        })
    }
}

pub struct RTTransmissiveShadowPassData {
    sun_direction: [f32; 3],
    sun_angular_radius: f32,
    sample_count: u32,
}

impl Pass for RTTransmissiveShadowPass {
    type PassData = RTTransmissiveShadowPassData;

    fn name(&self) -> String {
        String::from("rt_transmissive_shadow")
    }

    fn is_enabled(&self, context: &FrameDataContext) -> bool {
        context.render_settings.shadow_enabled.value
    }

    fn prepare_data(
        &self,
        context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let settings = context.render_settings;

        Ok(RTTransmissiveShadowPassData {
            sun_direction: (-context.render_snapshot.global_shadows_direction).to_array(),
            sun_angular_radius: settings.shadow_softness.value.to_radians(),
            sample_count: settings.shadow_samples.value.round().max(1.0) as u32,
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
                self.transmittance_image,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.scene_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.entity_buffer,
                AccessFlags::SHADER_READ,
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
        buffer_scope: &BufferResourceScope,
        data: Self::PassData,
    ) -> Result<()> {
        let depth_image = image_scope.get_physical_image(self.depth_image);
        let normal_image = image_scope.get_physical_image(self.normal_image);
        let transmittance_image = image_scope.get_physical_image(self.transmittance_image);
        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);
        let entity_buffer = buffer_scope.get_physical_buffer(self.entity_buffer);

        let depth_descriptor_id = depth_image
            .descriptors
            .full
            .expect("RTTransmissiveShadow depth image must have a sampled descriptor");

        let normal_descriptor_id = normal_image
            .descriptors
            .full
            .expect("RTTransmissiveShadow normal image must have a sampled descriptor");

        let transmittance_storage_id = transmittance_image
            .descriptors
            .storage_mips
            .as_ref()
            .and_then(|mips| mips.first().copied())
            .expect("RTTransmissiveShadow transmittance image must have a storage descriptor");

        let width = transmittance_image.extent.width;
        let height = transmittance_image.extent.height;

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);
        context.push_constants(
            self.pipeline_layout,
            &RTTransmissiveShadowPushConstants::create(
                data.sun_direction,
                scene_buffer,
                entity_buffer,
                context.resource_buffers.mesh_buffer,
                context.resource_buffers.submesh_buffer,
                context.resource_buffers.material_buffer,
                depth_descriptor_id,
                normal_descriptor_id,
                transmittance_storage_id,
                ResourceId::from(context.frame_index.value),
                data.sun_angular_radius,
                data.sample_count,
                context.frame_number,
            ),
        );
        context.dispatch_2d(width, height);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        Ok(())
    }
}
