use render_graph::VirtualData;
use render_graph::Pass;
use render_graph::FrameContext;
use crate::render::pass_resources::pass_resources::PassResources;
use anyhow::{bail, Context, Result};
use ash::vk::{AccessFlags, DeviceSize, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use crate::render::pass::skinning::gpu::bone_transform_gpu::BoneTransformGPU;
use render_snapshot::RenderSnapshot;
use std::sync::Arc;
use tracing::info;
use gpu::ResourceFactories;
use gpu::GpuSize;
use crate::render::pass::skinning::skinning_push_constants::SkinningPushConstants;
use render_graph::PassResourceDeclaration;
use render_graph::PrepareScopes;
use render_graph::RecordScopes;
use render_graph::DataResourceScope;
use render_graph::VirtualBuffer;
use resource_residency::ResRef;
use resource_residency::ResourceProvider;
use resource_store::SkeletonBackend;
use index_allocator::ResourceId;
use gpu::PipelineLayoutType;
use crate::render::pass::skinning::gpu::skinning_instance_gpu::SkinningInstanceGPU;
use crate::render::pass::skinning::gpu::skinning_pose_gpu::SkinningPoseGPU;
use pipeline_store::ComputePipelineConfig;
use crate::resource_manifest::shaders;

pub struct SkinningPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    skinning_instance: VirtualBuffer,
    bone_transform: VirtualBuffer,

    animation_buffer: VirtualBuffer,
    animation_frame_buffer: VirtualBuffer,
    skeleton_buffer: VirtualBuffer,
    skeleton_bone_buffer: VirtualBuffer,

    render_snapshot: VirtualData<RenderSnapshot>,

    skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
}

impl SkinningPass {
    pub fn create(
        resources: &PassResources,
        skinning_instance: VirtualBuffer,
        bone_transform: VirtualBuffer,
        render_snapshot: VirtualData<RenderSnapshot>,
        skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::SKINNING_COMP,
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

            skinning_instance,
            bone_transform,

            animation_buffer: resources.resource_buffer_handles.animation_buffer,
            animation_frame_buffer: resources.resource_buffer_handles.animation_frame_buffer,
            skeleton_buffer: resources.resource_buffer_handles.skeleton_buffer,
            skeleton_bone_buffer: resources.resource_buffer_handles.skeleton_bone_buffer,

            render_snapshot,

            skeleton_provider,
        })
    }
}

pub struct SkinningPassData {
    instance_count: u32,
}

impl Pass for SkinningPass {
    type PassData = SkinningPassData;

    fn name(&self) -> String {
        String::from("skinning")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_snapshot)
            .write_buffer(
                self.skinning_instance,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .read_buffer(
                self.skinning_instance,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.bone_transform,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.skeleton_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.skeleton_bone_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.animation_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.animation_frame_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn prepare_data(
        &self,
        scopes: &mut PrepareScopes,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let render_snapshot = scopes.data.get(self.render_snapshot);

        let mut bone_transform_count = 0;
        let mut instances = Vec::new();

        for entity in render_snapshot.entities.iter() {
            let Some(animation) = entity.animation.as_ref() else {
                continue;
            };

            let bone_count = self.skeleton_provider
                .with_resource(ResourceId::from(animation.skeleton_id), |skeleton| skeleton.bones_allocation.size)
                .context("Animated entity skeleton is not resident")?;

            instances.push(SkinningInstanceGPU::new(
                animation.skeleton_id,
                bone_transform_count,
                bone_transform_count + bone_count,
                SkinningPoseGPU::create(&animation.pose),
                SkinningPoseGPU::create(&animation.previous_pose),
            ));

            bone_transform_count += 2 * bone_count;
        }

        self.skinning_instance.stage_slice(scopes.buffer, &instances)?;

        self.bone_transform.reserve_region(
            scopes.buffer,
            bone_transform_count as DeviceSize * BoneTransformGPU::SIZE,
        )?;

        Ok(Self::PassData {
            instance_count: instances.len() as u32,
        })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        scopes: &RecordScopes,
        data: Self::PassData,
    ) -> Result<()> {
        let animation_frame_buffer = scopes.buffer.get_physical_buffer(self.animation_frame_buffer);
        let skeleton_buffer = scopes.buffer.get_physical_buffer(self.skeleton_buffer);
        let skeleton_bone_buffer = scopes.buffer.get_physical_buffer(self.skeleton_bone_buffer);
        let animation_buffer = scopes.buffer.get_physical_buffer(self.animation_buffer);

        let instance_count = data.instance_count;
        if instance_count == 0 {
            return Ok(());
        }

        let bone_transform = scopes.buffer.get_physical_buffer(self.bone_transform);
        let skinning_instance = scopes.buffer.get_physical_buffer(self.skinning_instance);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &SkinningPushConstants::create(
                skinning_instance.range,
                animation_buffer.range,
                animation_frame_buffer.range,
                skeleton_buffer.range,
                skeleton_bone_buffer.range,
                bone_transform.range,
                instance_count,
            ),
        );

        context.dispatch(instance_count);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("{} destroyed", self.name());

        Ok(())
    }
}
