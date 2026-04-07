use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, DependencyFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::ids::{FrameIndex, SliceIndex};
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::skinning::skinning_push_constants::SkinningPushConstants;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::dynamic::skinning::bone_transform_handler::BoneTransformHandler;
use crate::resources::dynamic::skinning::skinning_buffer::SkinningInstanceGPU;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};

pub struct SkinningPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    bone_transform_handler: Arc<BoneTransformHandler>,
}

impl SkinningPass {
    pub fn create(
        compute_pipeline_provider: &ResourceProvider<ComputePipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        bone_transform_handler: Arc<BoneTransformHandler>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: String::from("shaders/skinning/skinning.comp.spv"),
            fn_name: String::from("main"),
        };

        let _handle = compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = compute_pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire ComputePipeline");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            bone_transform_handler,
        })
    }
}

pub struct SkinningPassData {
    instances: Vec<SkinningInstanceGPU>,
}

impl Pass for SkinningPass {
    type PassData = SkinningPassData;
    type Statistics = ();

    fn name(&self) -> String {
        String::from("skinning")
    }
    
    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(&self, context: &FrameDataContext) -> Result<Self::PassData> {
        let instances = context.render_snapshot.entities.iter()
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

        Ok(Self::PassData {
            instances,
        })
    }

    fn record_commands(&self, context: &PassContext, _resource_registry: &ResourceRegistry, data: Self::PassData) -> Result<()> {
        let instance_count = data.instances.len() as u32;
        if instance_count == 0 {
            return Ok(());
        }

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        let barrier = self.bone_transform_handler.skinning_instance_buffer
            .slice_at(SliceIndex::ZERO)
            .stage(&data.instances, AccessFlags::SHADER_READ)?;

        context.pipeline_barrier(
            PipelineStageFlags::HOST,
            PipelineStageFlags::COMPUTE_SHADER,
            DependencyFlags::empty(),
            &[],
            &[barrier],
            &[],
        );

        context.push_constants(
            self.pipeline_layout,
            &SkinningPushConstants::create(
                self.bone_transform_handler.skinning_instance_buffer.slice_at(SliceIndex::ZERO).device_address(),
                context.resource_buffers.animation_buffer,
                context.resource_buffers.animation_frame_buffer,
                context.resource_buffers.skeleton_buffer,
                context.resource_buffers.skeleton_bone_buffer,
                self.bone_transform_handler.bone_transform_buffer.slice_at(SliceIndex::ZERO).device_address(),
                instance_count,
            ),
        );

        context.dispatch(instance_count);

        context.pipeline_barrier(
            PipelineStageFlags::COMPUTE_SHADER,
            PipelineStageFlags::VERTEX_SHADER,
            DependencyFlags::empty(),
            &[],
            &[
                self.bone_transform_handler.bone_transform_buffer.as_view().barrier(
                    AccessFlags::SHADER_WRITE,
                    AccessFlags::SHADER_READ,
                ),
            ],
            &[],
        );

        Ok(())
    }

    fn statistics(&self, _frame_index: FrameIndex) -> Self::Statistics {
        ()
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("{} destroyed", self.name());

        Ok(())
    }
}
