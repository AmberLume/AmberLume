use render_graph::ReadbackScope;
use render_graph::VirtualData;
use render_snapshot::RenderSnapshot;
use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::selection_mask::selection_mask_push_constants::SelectionMaskPushConstants;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::HeapAllocator;
use render_graph::VirtualImage;
use render_graph::VirtualBuffer;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::ImageResourceScope;
use gpu::PipelineLayoutType;
use crate::resource_manifest::shaders;
use pipeline_store::ComputePipelineConfig;
use resource_residency::ResRef;
use anyhow::{bail, Result};
use ash::vk::{
    AccessFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags,
};
use std::sync::Arc;

pub struct SelectionMaskPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    entity_id_image: VirtualImage,
    mask_image: VirtualImage,
    entity_outline_buffer: VirtualBuffer,

    render_snapshot: VirtualData<RenderSnapshot>,
}

impl SelectionMaskPass {
    pub const MASK_SCALE: i32 = 8;

    pub fn create(
        resources: &PassResources,
        entity_id_image: VirtualImage,
        mask_image: VirtualImage,
        entity_outline_buffer: VirtualBuffer,
        render_snapshot: VirtualData<RenderSnapshot>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::SELECTION_MASK_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config)?;
        let Some(pipeline) = resources.compute_pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire ComputePipeline for SelectionMask");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            entity_id_image,
            mask_image,
            entity_outline_buffer,

            render_snapshot,
        })
    }
}

impl Pass for SelectionMaskPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("selection_mask")
    }

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        data_scope.get(self.render_snapshot).entities.iter().any(|entity| entity.outline[3] > 0.0)
    }

    fn prepare_data(
        &self,
        _data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_snapshot)
            .read_image(
                self.entity_id_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_image(
                self.mask_image,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .read_buffer(
                self.entity_outline_buffer,
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
        _data: Self::PassData,
    ) -> Result<()> {
        let entity_id_image = image_scope.get_physical_image(self.entity_id_image);
        let mask_image = image_scope.get_physical_image(self.mask_image);
        let entity_outline_buffer = buffer_scope.get_physical_buffer(self.entity_outline_buffer);

        let entity_id_texture = entity_id_image
            .descriptors
            .full
            .expect("SelectionMask entity id image must have a sampled descriptor");

        let mask_storage_id = mask_image
            .descriptors
            .storage_mips
            .as_ref()
            .and_then(|mips| mips.first().copied())
            .expect("SelectionMask mask image must have a storage descriptor");

        let width = mask_image.extent.width;
        let height = mask_image.extent.height;

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);
        context.push_constants(
            self.pipeline_layout,
            &SelectionMaskPushConstants::create(
                entity_outline_buffer.range,
                entity_id_texture.inner,
                mask_storage_id.inner,
                width,
                height,
                Self::MASK_SCALE,
            ),
        );

        context.dispatch_2d(width, height);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        Ok(())
    }
}
