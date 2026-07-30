use gpu::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::tlas_instances::tlas_instances_push_constants::TLASInstancesPushConstants;
use crate::render::ray_tracing::ray_tracing::RayTracing;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use gpu::PipelineLayoutType;
use crate::resources::resource_manifest::shaders;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::store::providers::res_ref::ResRef;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;

pub struct TLASInstancesPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    ray_tracing: Arc<RayTracing>,
    entity_buffer: VirtualBuffer,
    instances: VirtualBuffer,
}

impl TLASInstancesPass {
    pub fn create(
        resources: &PassResources,
        ray_tracing: Arc<RayTracing>,
        entity_buffer: VirtualBuffer,
        instances: VirtualBuffer,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::TLAS_INSTANCES_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline for TLASInstances");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            ray_tracing,
            entity_buffer,
            instances,
        })
    }
}

pub struct TLASInstancesPassData {
    entity_count: usize,
}

impl Pass for TLASInstancesPass {
    type PassData = TLASInstancesPassData;

    fn name(&self) -> String {
        String::from("tlas_instances")
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
        Ok(TLASInstancesPassData {
            entity_count: context.render_snapshot.entities.len(),
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .read_buffer(
                self.entity_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.instances,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn record_commands(
        &self,
        context: &PassContext,
        _image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        data: Self::PassData,
    ) -> Result<()> {
        if data.entity_count == 0 {
            return Ok(());
        }

        let entity_buffer = buffer_scope.get_physical_buffer(self.entity_buffer);
        let instances = buffer_scope.get_physical_buffer(self.instances);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);
        context.push_constants(
            self.pipeline_layout,
            &TLASInstancesPushConstants::create(
                entity_buffer.device_address,
                self.ray_tracing.blas.addresses_buffer.device_address,
                instances.device_address,
                context.resource_buffers.mesh_buffer,
                context.resource_buffers.submesh_buffer,
                context.resource_buffers.material_buffer,
                data.entity_count as u32,
            ),
        );
        context.dispatch(data.entity_count as u32);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        Ok(())
    }
}
