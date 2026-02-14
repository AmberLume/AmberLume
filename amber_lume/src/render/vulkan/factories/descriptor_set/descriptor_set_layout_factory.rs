use std::sync::Arc;
use ash::Device;
use anyhow::Result;
use ash::vk::{DescriptorBindingFlags, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutBindingFlagsCreateInfo, DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo, DescriptorType, ShaderStageFlags};
use tracing::info;
use crate::render::vulkan::debug_utils::DebugUtils;

pub struct DescriptorSetLayoutBindingDescription {
    pub binding: u32,
    pub binding_flags: DescriptorBindingFlags,
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
    pub stage_flags: ShaderStageFlags,
}

pub struct DescriptorSetLayoutFactory {
    device: Device,
    debug_utils: Arc<DebugUtils>,
}

impl DescriptorSetLayoutFactory {
    pub fn create(
        device: Device,
        debug_utils: Arc<DebugUtils>,
    ) -> Self {
        Self {
            device,
            debug_utils,
        }
    }

    fn create_descriptor_set_layout(
        &self,
        label: &str,
        bindings_descriptions: &[DescriptorSetLayoutBindingDescription],
    ) -> Result<DescriptorSetLayout> {
        let mut bindings = Vec::with_capacity(bindings_descriptions.len());
        let mut binding_flags = Vec::with_capacity(bindings_descriptions.len());

        for binding_description in bindings_descriptions {
            let layout_binding = Self::create_descriptor_set_layout_binding(&binding_description);

            bindings.push(layout_binding);
            binding_flags.push(binding_description.binding_flags)
        }

        let mut binding_flags_create_info = DescriptorSetLayoutBindingFlagsCreateInfo::default()
            .binding_flags(&binding_flags);

        let layout_info = DescriptorSetLayoutCreateInfo::default()
            .push_next(&mut binding_flags_create_info)
            .bindings(&bindings)
            .flags(DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);

        let descriptor_set_layout = unsafe { self.device.create_descriptor_set_layout(&layout_info, None)? };

        self.debug_utils.label(descriptor_set_layout, &format!("descriptor_set_layout_{}", label));

        Ok(descriptor_set_layout)
    }

    fn create_descriptor_set_layout_binding<'a>(
        description: &DescriptorSetLayoutBindingDescription,
    ) -> DescriptorSetLayoutBinding<'a> {
        DescriptorSetLayoutBinding::default()
            .binding(description.binding)
            .descriptor_type(description.descriptor_type)
            .descriptor_count(description.descriptor_count)
            .stage_flags(description.stage_flags)
    }

    fn destroy(
        &self,
        descriptor_set_layout: DescriptorSetLayout,
    ) -> Result<()> {
        unsafe { self.device.destroy_descriptor_set_layout(descriptor_set_layout, None) }

        info!("DescriptorSetLayout destroyed");

        Ok(())
    }
}