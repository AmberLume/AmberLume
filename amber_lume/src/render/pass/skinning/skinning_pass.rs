use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::skinning::skinning_push_constants::SkinningPushConstants;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
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
}

impl SkinningPass {
    pub fn create(
        resources: &PassResources,
        skinning_instance: VirtualBuffer,
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

            skinning_instance,
            bone_transform,
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
    
    fn is_enabled(&self, _context: &FrameDataContext) -> bool {
        true
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
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
        context: &FrameDataContext,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
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

        self.skinning_instance.stage_slice(buffer_scope, allocator, &instances)?;

        Ok(Self::PassData {
            instance_count: instances.len() as u32,
        })
    }

    fn record_commands(&self, context: &PassContext, _image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope, data: Self::PassData) -> Result<()> {
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
