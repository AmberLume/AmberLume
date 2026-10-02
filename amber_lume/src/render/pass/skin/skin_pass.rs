use crate::render::pass::skin::gpu::skin_target_gpu::SkinTargetGPU;
use crate::render::pass::skin::skin_push_constants::SkinPushConstants;
use crate::render::pass_resources::pass_resources::PassResources;
use crate::resource_manifest::shaders;
use anyhow::{bail, Context, Result};
use ash::vk::{AccessFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use gpu::PipelineLayoutType;
use gpu::ResourceFactories;
use parking_lot::Mutex;
use pipeline_store::ComputePipelineConfig;
use render_graph::DataResourceScope;
use render_graph::FrameContext;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::PrepareScopes;
use render_graph::RecordScopes;
use render_graph::VirtualBuffer;
use render_graph::VirtualData;
use render_snapshot::RenderSnapshot;
use resource_residency::ResRef;
use resource_store::FrameSliceIndex;
use resource_store::MeshBackend;
use index_allocator::ResourceId;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::info;

pub struct SkinPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    render_snapshot: VirtualData<RenderSnapshot>,
    skin_slice_index: VirtualData<FrameSliceIndex>,

    skinning_instance: VirtualBuffer,
    skin_target: VirtualBuffer,
    bone_transform: VirtualBuffer,
    vertex_position_buffer: VirtualBuffer,
    vertex_normal_tangent_buffer: VirtualBuffer,
    mesh_vertex_skin_buffer: VirtualBuffer,

    mesh_backend: Arc<MeshBackend>,

    written_instances: Mutex<HashSet<u32>>,
}

impl SkinPass {
    pub fn create(
        resources: &PassResources,
        skin_target: VirtualBuffer,
        skinning_instance: VirtualBuffer,
        bone_transform: VirtualBuffer,
        render_snapshot: VirtualData<RenderSnapshot>,
        skin_slice_index: VirtualData<FrameSliceIndex>,
        mesh_backend: Arc<MeshBackend>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::SKIN_COMP,
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

            render_snapshot,
            skin_slice_index,

            skinning_instance,
            skin_target,
            bone_transform,
            vertex_position_buffer: resources.resource_buffer_handles.vertex_position_buffer,
            vertex_normal_tangent_buffer: resources.resource_buffer_handles.vertex_normal_tangent_buffer,
            mesh_vertex_skin_buffer: resources.resource_buffer_handles.mesh_vertex_skin_buffer,

            mesh_backend,

            written_instances: Mutex::new(HashSet::new()),
        })
    }
}

pub struct SkinPassData {
    target_count: u32,
    vertex_count: u32,
}

impl Pass for SkinPass {
    type PassData = SkinPassData;

    fn name(&self) -> String {
        String::from("skin")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_snapshot)
            .consume(self.skin_slice_index)
            .write_buffer(
                self.skin_target,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .read_buffer(
                self.skin_target,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.skinning_instance,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.bone_transform,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.mesh_vertex_skin_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.vertex_position_buffer,
                AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.vertex_normal_tangent_buffer,
                AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn prepare_data(
        &self,
        scopes: &mut PrepareScopes,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let render_snapshot = scopes.data.get(self.render_snapshot);

        let skin_slice_index = *scopes.data.get(self.skin_slice_index);

        let mut written_instances = self.written_instances.lock();
        let mut current_instances = HashSet::new();

        let mut targets = Vec::new();
        let mut vertex_count = 0;

        let animated_entities = render_snapshot.entities
            .iter()
            .filter_map(|entity| entity.animation.as_ref().map(|animation| (entity, animation)));

        for (instance_index, (entity, animation)) in animated_entities.enumerate() {
            let fresh = !written_instances.contains(&animation.skin_id);

            let slices = if fresh {
                vec![skin_slice_index.current, skin_slice_index.previous]
            } else {
                vec![skin_slice_index.current]
            };

            let (source_vertex_offset, source_skin_offset) = self.mesh_backend
                .with_mesh(ResourceId::from(entity.mesh_id), |source| {
                    source.skeletal
                        .as_ref()
                        .map(|skeletal| (source.vertices_allocation.offset, skeletal.vertex_skins_allocation.offset))
                })
                .context("Skin source mesh is not resident")?
                .context("Skin source mesh has no vertex skins")?;

            self.mesh_backend
                .with_instance(ResourceId::from(animation.skin_id), |skin| {
                    for slice in slices {
                        targets.push(SkinTargetGPU::new(
                            source_vertex_offset,
                            source_skin_offset,
                            skin.vertices_allocation.offset + slice * skin.vertex_slice_stride,
                            instance_index as u32,
                            vertex_count,
                        ));

                        vertex_count += skin.vertex_slice_stride;
                    }
                })
                .context("Skin is not resident")?;

            current_instances.insert(animation.skin_id);
        }

        *written_instances = current_instances;

        self.skin_target.stage_slice(scopes.buffer, &targets)?;

        Ok(Self::PassData {
            target_count: targets.len() as u32,
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

        let skin_target = scopes.buffer.get_physical_buffer(self.skin_target);
        let skinning_instance = scopes.buffer.get_physical_buffer(self.skinning_instance);
        let bone_transform = scopes.buffer.get_physical_buffer(self.bone_transform);
        let vertex_position_buffer = scopes.buffer.get_physical_buffer(self.vertex_position_buffer);
        let vertex_normal_tangent_buffer = scopes.buffer.get_physical_buffer(self.vertex_normal_tangent_buffer);
        let mesh_vertex_skin_buffer = scopes.buffer.get_physical_buffer(self.mesh_vertex_skin_buffer);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &SkinPushConstants::create(
                skin_target.range,
                skinning_instance.range,
                bone_transform.range,
                vertex_position_buffer.range,
                vertex_normal_tangent_buffer.range,
                mesh_vertex_skin_buffer.range,
                data.target_count,
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
