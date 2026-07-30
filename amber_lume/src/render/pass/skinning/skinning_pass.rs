use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, DependencyFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use gpu::SliceIndex;
use gpu::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::skinning::skinning_push_constants::SkinningPushConstants;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::resources::store::providers::res_ref::ResRef;
use gpu::PipelineLayoutType;
use crate::resources::skinning::bone_transform_handler::BoneTransformHandler;
use crate::resources::skinning::skinning_buffer::SkinningInstanceGPU;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::resource_manifest::shaders;

pub struct SkinningPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    bone_transform_handler: Arc<BoneTransformHandler>,
    bone_transform: VirtualBuffer,
}

impl SkinningPass {
    pub fn create(
        resources: &PassResources,
        bone_transform_handler: Arc<BoneTransformHandler>,
        bone_transform: VirtualBuffer,
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

            bone_transform_handler,
            bone_transform,
        })
    }
}

pub struct SkinningPassData {
    instances: Vec<SkinningInstanceGPU>,
}

impl Pass for SkinningPass {
    type PassData = SkinningPassData;

    fn name(&self) -> String {
        String::from("skinning")
    }
    
    fn is_enabled(&self) -> bool {
        true
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration.write_buffer(
            self.bone_transform,
            AccessFlags::SHADER_WRITE,
            PipelineStageFlags::COMPUTE_SHADER,
        );
    }

    fn prepare_data(
        &self, 
        context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
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

    fn record_commands(&self, context: &PassContext, _image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope, data: Self::PassData) -> Result<()> {
        let instance_count = data.instances.len() as u32;
        if instance_count == 0 {
            return Ok(());
        }

        let bone_transform = buffer_scope.get_physical_buffer(self.bone_transform);

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
