use render_graph::ReadbackScope;
use render_graph::VirtualData;
use render_graph::Pass;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, DeviceAddress, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use render_snapshot::RenderSnapshot;
use std::sync::Arc;
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::skinning::skinning_push_constants::SkinningPushConstants;
use render_graph::PassResourceDeclaration;
use render_graph::ImageResourceScope;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::HeapAllocator;
use render_graph::VirtualBuffer;
use resource_residency::ResRef;
use gpu::PipelineLayoutType;
use crate::render::frame_data::skinning_instance_gpu::SkinningInstanceGPU;
use pipeline_store::ComputePipelineConfig;
use crate::resource_manifest::shaders;

pub struct SkinningPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    skinning_instance: VirtualBuffer,
    bone_transform: VirtualBuffer,

    animation_buffer: DeviceAddress,
    animation_frame_buffer: DeviceAddress,
    skeleton_buffer: DeviceAddress,
    skeleton_bone_buffer: DeviceAddress,

    render_snapshot: VirtualData<RenderSnapshot>,
}

impl SkinningPass {
    pub fn create(
        resources: &PassResources,
        skinning_instance: VirtualBuffer,
        bone_transform: VirtualBuffer,
        render_snapshot: VirtualData<RenderSnapshot>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::SKINNING_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = resources.compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            skinning_instance,
            bone_transform,

            animation_buffer: resources.resource_buffers.animation_buffer,
            animation_frame_buffer: resources.resource_buffers.animation_frame_buffer,
            skeleton_buffer: resources.resource_buffers.skeleton_buffer,
            skeleton_bone_buffer: resources.resource_buffers.skeleton_bone_buffer,

            render_snapshot,
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
            );
    }

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let render_snapshot = data_scope.get(self.render_snapshot);

        let instances = render_snapshot.entities.iter()
            .filter_map(|entity| {
                entity.animation.as_ref().map(|animation| {
                    SkinningInstanceGPU::new(
                        animation.animation_id,
                        animation.skeleton_id,
                        animation.bone_transform_offset,
                        animation.time,
                        animation.previous_animation_id,
                        animation.previous_time,
                        animation.blend_factor,
                    )
                })
            })
            .collect::<Vec<_>>();

        self.skinning_instance.stage_slice(buffer_scope, allocator, &instances)?;

        Ok(Self::PassData {
            instance_count: instances.len() as u32,
        })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        _image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        data: Self::PassData,
    ) -> Result<()> {
        let instance_count = data.instance_count;
        if instance_count == 0 {
            return Ok(());
        }

        let bone_transform = buffer_scope.get_physical_buffer(self.bone_transform);
        let skinning_instance = buffer_scope.get_physical_buffer(self.skinning_instance);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &SkinningPushConstants::create(
                skinning_instance.device_address,
                self.animation_buffer,
                self.animation_frame_buffer,
                self.skeleton_buffer,
                self.skeleton_bone_buffer,
                bone_transform,
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
