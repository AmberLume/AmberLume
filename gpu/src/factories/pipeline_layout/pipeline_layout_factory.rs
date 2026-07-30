use std::slice::from_ref;
use std::sync::Arc;
use ash::Device;
use ash::vk::{DescriptorSetLayout, PipelineLayout, PipelineLayoutCreateInfo, PushConstantRange, ShaderStageFlags};
use tracing::info;
use crate::utils::debug_utils::DebugUtils;
use anyhow::Result;

pub struct PipelineLayoutFactory {
    device: Device,
    debug_utils: Arc<DebugUtils>,
}

impl PipelineLayoutFactory {
    pub const PUSH_CONSTANTS_SIZE: u32 = 128;

    pub fn create(
        device: Device,
        debug_utils: Arc<DebugUtils>,
    ) -> Self {
        Self {
            device,
            debug_utils,
        }
    }

    pub fn create_pipeline_layout(
        &self,
        label: &str,
        descriptor_set_layout: DescriptorSetLayout,
    ) -> Result<PipelineLayout> {
        let push_constant_range = PushConstantRange::default()
            .stage_flags(ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT | ShaderStageFlags::COMPUTE)
            .offset(0)
            .size(Self::PUSH_CONSTANTS_SIZE);

        let layout_info = PipelineLayoutCreateInfo::default()
            .set_layouts(from_ref(&descriptor_set_layout))
            .push_constant_ranges(from_ref(&push_constant_range));

        let pipeline_layout = unsafe { self.device.create_pipeline_layout(&layout_info, None)? };

        self.debug_utils.label(pipeline_layout, &format!("pipeline_layout_{}", label));

        Ok(pipeline_layout)
    }

    pub fn destroy_pipeline_layout(&self, pipeline_layout: PipelineLayout) {
        unsafe { self.device.destroy_pipeline_layout(pipeline_layout, None) }

        info!("PipelineLayout destroyed");
    }
}
