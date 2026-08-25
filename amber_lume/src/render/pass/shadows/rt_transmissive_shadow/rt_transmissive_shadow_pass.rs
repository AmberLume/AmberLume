use render_graph::ReadbackScope;
use render_graph::VirtualData;
use settings::RenderSettings;
use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::shadows::rt_transmissive_shadow::rt_transmissive_shadow_push_constants::RTTransmissiveShadowPushConstants;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::VirtualAccelerationStructure;
use render_graph::VirtualBuffer;
use render_graph::VirtualImage;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::ImageResourceScope;
use gpu::PipelineLayoutType;
use crate::resource_manifest::shaders;
use pipeline_store::ComputePipelineConfig;
use resource_residency::ResRef;
use anyhow::{bail, Result};
use ash::vk::{
    AccessFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout,
    PipelineStageFlags,
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

    mesh_buffer: VirtualBuffer,
    submesh_buffer: VirtualBuffer,
    material_buffer: VirtualBuffer,

    render_settings: VirtualData<RenderSettings>,
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
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::RT_TRANSMISSIVE_SHADOW_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config)?;
        let Some(pipeline) = resources.compute_pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire ComputePipeline for RTTransmissiveShadow");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources
                .pipeline_layout_registry
                .get(PipelineLayoutType::General),

            depth_image,
            normal_image,
            transmittance_image,
            scene_buffer,
            entity_buffer,
            tlas,

            mesh_buffer: resources.resource_buffer_handles.mesh_buffer,
            submesh_buffer: resources.resource_buffer_handles.submesh_buffer,
            material_buffer: resources.resource_buffer_handles.material_buffer,
        
            render_settings,
        })
    }
}

pub struct RTTransmissiveShadowPassData {
    sun_angular_radius: f32,
    sample_count: u32,
}

impl Pass for RTTransmissiveShadowPass {
    type PassData = RTTransmissiveShadowPassData;

    fn name(&self) -> String {
        String::from("rt_transmissive_shadow")
    }

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        data_scope.get(self.render_settings).shadow_enabled.value
    }

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let settings = data_scope.get(self.render_settings);

        Ok(RTTransmissiveShadowPassData {
            sun_angular_radius: settings.shadow_softness.value.to_radians(),
            sample_count: settings.shadow_samples.value.round().max(1.0) as u32,
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_settings)
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
            )
            .read_buffer(
                self.submesh_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.mesh_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.material_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        data: Self::PassData,
    ) -> Result<()> {
        let mesh_buffer = buffer_scope.get_physical_buffer(self.mesh_buffer);
        let material_buffer = buffer_scope.get_physical_buffer(self.material_buffer);
        let submesh_buffer = buffer_scope.get_physical_buffer(self.submesh_buffer);

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
                scene_buffer.range,
                entity_buffer.range,
                mesh_buffer.range,
                submesh_buffer.range,
                material_buffer.range,
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
