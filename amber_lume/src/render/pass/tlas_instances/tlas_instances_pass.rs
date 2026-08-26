use render_graph::VirtualData;
use render_snapshot::RenderSnapshot;
use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::tlas_instances::tlas_instances_push_constants::TLASInstancesPushConstants;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::VirtualBuffer;
use render_graph::PrepareScopes;
use render_graph::RecordScopes;
use render_graph::DataResourceScope;
use gpu::PipelineLayoutType;
use crate::resource_manifest::shaders;
use pipeline_store::ComputePipelineConfig;
use resource_residency::ResRef;
use anyhow::{bail, Result};
use ash::vk::{AccelerationStructureInstanceKHR, AccessFlags, DeviceSize, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::mem::size_of;
use std::sync::Arc;

pub struct TLASInstancesPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    entity_buffer: VirtualBuffer,
    blas_addresses: VirtualBuffer,
    instances: VirtualBuffer,
    mesh_buffer: VirtualBuffer,
    submesh_buffer: VirtualBuffer,
    material_buffer: VirtualBuffer,

    render_snapshot: VirtualData<RenderSnapshot>,
}

impl TLASInstancesPass {
    pub fn create(
        resources: &PassResources,
        entity_buffer: VirtualBuffer,
        blas_addresses: VirtualBuffer,
        instances: VirtualBuffer,
        render_snapshot: VirtualData<RenderSnapshot>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::TLAS_INSTANCES_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config)?;
        let Some(pipeline) = resources.compute_pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire ComputePipeline for TLASInstances");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            entity_buffer,
            blas_addresses,
            instances,
            mesh_buffer: resources.resource_buffer_handles.mesh_buffer,
            submesh_buffer: resources.resource_buffer_handles.submesh_buffer,
            material_buffer: resources.resource_buffer_handles.material_buffer,

            render_snapshot,
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

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn prepare_data(
        &self,
        scopes: &mut PrepareScopes,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let render_snapshot = scopes.data.get(self.render_snapshot);
        
        let entity_count = render_snapshot.entities.len();

        self.instances.reserve_region(
            scopes.buffer,
            entity_count as DeviceSize * size_of::<AccelerationStructureInstanceKHR>() as DeviceSize,
        )?;

        Ok(TLASInstancesPassData {
            entity_count,
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_snapshot)
            .read_buffer(
                self.entity_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.blas_addresses,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.instances,
                AccessFlags::SHADER_WRITE,
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
        scopes: &RecordScopes,
        data: Self::PassData,
    ) -> Result<()> {
        let mesh_buffer = scopes.buffer.get_physical_buffer(self.mesh_buffer);
        let material_buffer = scopes.buffer.get_physical_buffer(self.material_buffer);
        let submesh_buffer = scopes.buffer.get_physical_buffer(self.submesh_buffer);

        if data.entity_count == 0 {
            return Ok(());
        }

        let entity_buffer = scopes.buffer.get_physical_buffer(self.entity_buffer);
        let blas_addresses = scopes.buffer.get_physical_buffer(self.blas_addresses);
        let instances = scopes.buffer.get_physical_buffer(self.instances);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);
        context.push_constants(
            self.pipeline_layout,
            &TLASInstancesPushConstants::create(
                entity_buffer.range,
                blas_addresses.range,
                instances.range,
                mesh_buffer.range,
                submesh_buffer.range,
                material_buffer.range,
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
