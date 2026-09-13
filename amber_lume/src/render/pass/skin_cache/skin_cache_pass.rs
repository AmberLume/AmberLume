use render_graph::VirtualData;
use render_graph::Pass;
use render_graph::FrameContext;
use crate::render::pass_resources::pass_resources::PassResources;
use anyhow::{bail, Context, Result};
use ash::vk::{AccessFlags, DeviceSize, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use crate::render::pass::skin_cache::gpu::skin_cache_instance_gpu::SkinCacheInstanceGPU;
use crate::render::pass::skin_cache::skin_cache_push_constants::SkinCachePushConstants;
use render_snapshot::RenderSnapshot;
use std::sync::Arc;
use tracing::info;
use gpu::{GpuSize, ResourceFactories};
use gpu_data::MeshVertexAttributeGPU;
use gpu_data::MeshVertexGPU;
use index_allocator::ResourceId;
use render_graph::PassResourceDeclaration;
use render_graph::PrepareScopes;
use render_graph::RecordScopes;
use render_graph::DataResourceScope;
use render_graph::VirtualBuffer;
use resource_residency::ResRef;
use resource_residency::ResourceProvider;
use resource_store::MeshBackend;
use gpu::PipelineLayoutType;
use pipeline_store::ComputePipelineConfig;
use crate::resource_manifest::shaders;

pub struct SkinCachePass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    skin_cache_instance: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    bone_transform: VirtualBuffer,
    skin_cache_vertex: VirtualBuffer,
    skin_cache_vertex_attribute: VirtualBuffer,

    mesh_vertex_buffer: VirtualBuffer,
    mesh_vertex_attribute_buffer: VirtualBuffer,
    mesh_vertex_skin_buffer: VirtualBuffer,

    render_snapshot: VirtualData<RenderSnapshot>,

    mesh_provider: Arc<ResourceProvider<MeshBackend>>,
}

impl SkinCachePass {
    pub fn create(
        resources: &PassResources,
        skin_cache_instance: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        bone_transform: VirtualBuffer,
        skin_cache_vertex: VirtualBuffer,
        skin_cache_vertex_attribute: VirtualBuffer,
        render_snapshot: VirtualData<RenderSnapshot>,
        mesh_provider: Arc<ResourceProvider<MeshBackend>>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::SKIN_CACHE_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config)?;
        let Some(pipeline) = resources.compute_pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire ComputePipeline");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            skin_cache_instance,
            entity_buffer,
            bone_transform,
            skin_cache_vertex,
            skin_cache_vertex_attribute,

            mesh_vertex_buffer: resources.resource_buffer_handles.mesh_vertex_buffer,
            mesh_vertex_attribute_buffer: resources.resource_buffer_handles.mesh_vertex_attribute_buffer,
            mesh_vertex_skin_buffer: resources.resource_buffer_handles.mesh_vertex_skin_buffer,

            render_snapshot,

            mesh_provider,
        })
    }
}

pub struct SkinCachePassData {
    instance_count: u32,
    vertex_count: u32,
}

impl Pass for SkinCachePass {
    type PassData = SkinCachePassData;

    fn name(&self) -> String {
        String::from("skin_cache")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_snapshot)
            .write_buffer(
                self.skin_cache_instance,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .read_buffer(
                self.skin_cache_instance,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.bone_transform,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.mesh_vertex_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.mesh_vertex_attribute_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.mesh_vertex_skin_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.entity_buffer,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.skin_cache_vertex,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.skin_cache_vertex_attribute,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .publish_buffer(
                self.skin_cache_vertex,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::COMPUTE_SHADER,
            )
            .publish_buffer(
                self.skin_cache_vertex_attribute,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn prepare_data(
        &self,
        scopes: &mut PrepareScopes,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let render_snapshot = scopes.data.get(self.render_snapshot);

        let mut vertex_count = 0;

        for entity in render_snapshot.entities.iter() {
            if entity.animation.is_none() {
                continue;
            }

            vertex_count += self.mesh_provider
                .with_resource(ResourceId::from(entity.mesh_id), |mesh| mesh.vertices_allocation.size)
                .context("Animated entity mesh is not resident")?;
        }

        self.skin_cache_vertex.reserve_region(
            scopes.buffer,
            vertex_count as DeviceSize * MeshVertexGPU::SIZE,
        )?;
        self.skin_cache_vertex_attribute.reserve_region(
            scopes.buffer,
            vertex_count as DeviceSize * MeshVertexAttributeGPU::SIZE,
        )?;

        let skin_cache_vertex = scopes.buffer.get_physical_buffer(self.skin_cache_vertex);
        let skin_cache_vertex_attribute = scopes.buffer.get_physical_buffer(self.skin_cache_vertex_attribute);

        let mut skin_cache_offset = 0;
        let mut instances = Vec::new();

        for (entity_index, entity) in render_snapshot.entities.iter().enumerate() {
            let Some(animation) = entity.animation.as_ref() else {
                continue;
            };

            let instance = self.mesh_provider
                .with_resource(ResourceId::from(entity.mesh_id), |mesh| {
                    let instance = SkinCacheInstanceGPU::new(
                        entity_index as u32,
                        mesh.vertices_allocation.offset,
                        mesh.vertex_attributes_allocation.offset,
                        mesh.vertex_skins_allocation.unwrap().offset,
                        animation.bone_transform_offset,
                        skin_cache_offset,
                        skin_cache_vertex.range,
                        skin_cache_vertex_attribute.range,
                    );

                    skin_cache_offset += mesh.vertices_allocation.size;

                    instance
                })
                .context("Animated entity mesh is not resident")?;

            instances.push(instance);
        }

        self.skin_cache_instance.stage_slice(scopes.buffer, &instances)?;

        Ok(Self::PassData {
            instance_count: instances.len() as u32,
            vertex_count,
        })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        scopes: &RecordScopes,
        data: Self::PassData,
    ) -> Result<()> {
        if data.vertex_count == 0 {
            return Ok(());
        }
        
        let skin_cache_instance = scopes.buffer.get_physical_buffer(self.skin_cache_instance);
        let entity_buffer = scopes.buffer.get_physical_buffer(self.entity_buffer);
        let bone_transform = scopes.buffer.get_physical_buffer(self.bone_transform);
        let mesh_vertex_buffer = scopes.buffer.get_physical_buffer(self.mesh_vertex_buffer);
        let mesh_vertex_attribute_buffer = scopes.buffer.get_physical_buffer(self.mesh_vertex_attribute_buffer);
        let mesh_vertex_skin_buffer = scopes.buffer.get_physical_buffer(self.mesh_vertex_skin_buffer);
        let skin_cache_vertex = scopes.buffer.get_physical_buffer(self.skin_cache_vertex);
        let skin_cache_vertex_attribute = scopes.buffer.get_physical_buffer(self.skin_cache_vertex_attribute);
        
        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &SkinCachePushConstants::create(
                skin_cache_instance.range,
                entity_buffer.range,
                bone_transform.range,
                mesh_vertex_buffer.range,
                mesh_vertex_attribute_buffer.range,
                mesh_vertex_skin_buffer.range,
                skin_cache_vertex.range,
                skin_cache_vertex_attribute.range,
                data.instance_count,
                data.vertex_count,
            ),
        );

        context.dispatch(data.vertex_count);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("{} destroyed", self.name());

        Ok(())
    }
}
