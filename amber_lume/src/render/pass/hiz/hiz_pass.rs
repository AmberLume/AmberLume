use anyhow::{bail, Result};
use std::mem::size_of;
use ash::vk::{
    DeviceSize,
    AccessFlags, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout,
    PipelineStageFlags,
};
use std::sync::Arc;
use tracing::info;
use crate::limits::MAX_HIZ_MIPS;
use gpu::ResourceFactories;
use crate::render::pass::hiz::hiz_push_constants::HiZPushConstants;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::VirtualBuffer;
use render_graph::VirtualImage;
use render_graph::PrepareScopes;
use render_graph::RecordScopes;
use render_graph::DataResourceScope;
use gpu::PipelineLayoutType;
use crate::resource_manifest::shaders;
use pipeline_store::ComputePipelineConfig;
use resource_residency::ResRef;

pub struct HiZPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    depth_image: VirtualImage,
    hiz_image: VirtualImage,
    hiz_counter_buffer: VirtualBuffer,

    mip_count: u32,
}

impl HiZPass {
    pub fn create(
        resources: &PassResources,
        depth_image: VirtualImage,
        hiz_image: VirtualImage,
        hiz_counter_buffer: VirtualBuffer,
        mip_count: u32,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::HIZ_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config)?;
        let Some(pipeline) = resources.compute_pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire ComputePipeline for HiZ");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources
                .pipeline_layout_registry
                .get(PipelineLayoutType::General),

            depth_image,
            hiz_image,
            hiz_counter_buffer,

            mip_count,
        })
    }
}

impl Pass for HiZPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("hiz")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn prepare_data(
        &self,
        scopes: &mut PrepareScopes,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        self.hiz_counter_buffer.reserve_region(scopes.buffer, size_of::<u32>() as DeviceSize)?;

        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration.read_image(
            self.depth_image,
            ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            AccessFlags::SHADER_READ,
            PipelineStageFlags::COMPUTE_SHADER,
        );

        for mip in 0..self.mip_count {
            declaration.write_image_mip(
                self.hiz_image,
                mip,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
                PipelineStageFlags::COMPUTE_SHADER,
            );
        }

        declaration.write_buffer(
            self.hiz_counter_buffer,
            AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
            PipelineStageFlags::COMPUTE_SHADER,
        );
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        scopes: &RecordScopes,
        _data: Self::PassData,
    ) -> Result<()> {
        let depth_image = scopes.image.get_physical_image(self.depth_image);
        let hiz_image = scopes.image.get_physical_image(self.hiz_image);
        let hiz_counter_buffer = scopes.buffer.get_physical_buffer(self.hiz_counter_buffer);

        let depth_descriptor_id = depth_image
            .descriptors
            .full
            .expect("HiZ depth image must have a sampled descriptor");

        let storage_mips = hiz_image
            .descriptors
            .storage_mips
            .as_ref()
            .expect("HiZ image must have storage descriptors");

        let mut storage_ids = [0u32; MAX_HIZ_MIPS];
        for (index, resource_id) in storage_mips.iter().take(MAX_HIZ_MIPS).enumerate() {
            storage_ids[index] = resource_id.inner;
        }

        let width = depth_image.extent.width;
        let height = depth_image.extent.height;

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &HiZPushConstants::create(
                hiz_counter_buffer.range,
                depth_descriptor_id.inner,
                self.mip_count,
                width,
                height,
                storage_ids,
            ),
        );

        context.dispatch_groups((width + 63) / 64, (height + 63) / 64, 1);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("HiZPass destroyed");

        Ok(())
    }
}
