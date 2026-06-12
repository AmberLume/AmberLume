use std::sync::Arc;
use ash::Device;
use anyhow::Result;
use ash::vk::{DescriptorBindingFlags, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutBindingFlagsCreateInfo, DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo, DescriptorType, Sampler, ShaderStageFlags};
use tracing::info;
use crate::render::utils::debug_utils::DebugUtils;
use crate::resources::binding_layout::descriptor_set_manager::GlobalDescriptorSetBindings;

pub struct DescriptorSetLayoutBindingDescription {
    pub binding: GlobalDescriptorSetBindings,
    pub binding_flags: DescriptorBindingFlags,
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
    pub stage_flags: ShaderStageFlags,
    pub immutable_samplers: Vec<Sampler>,
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

    pub fn create_descriptor_set_layout(
        &self,
        label: &str,
        bindings_descriptions: &[DescriptorSetLayoutBindingDescription],
    ) -> Result<DescriptorSetLayout> {
        let mut bindings = Vec::with_capacity(bindings_descriptions.len());
        let mut binding_flags = Vec::with_capacity(bindings_descriptions.len());

        for binding_description in bindings_descriptions {
            let mut layout_binding = DescriptorSetLayoutBinding::default()
                .binding(binding_description.binding as u32)
                .descriptor_type(binding_description.descriptor_type)
                .stage_flags(binding_description.stage_flags);

            if binding_description.immutable_samplers.is_empty() {
                layout_binding = layout_binding.descriptor_count(binding_description.descriptor_count);
            } else {
                layout_binding = layout_binding.immutable_samplers(&binding_description.immutable_samplers);
            }

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

    pub fn destroy_descriptor_set_layout(
        &self,
        descriptor_set_layout: DescriptorSetLayout,
    ) {
        unsafe { self.device.destroy_descriptor_set_layout(descriptor_set_layout, None) }

        info!("DescriptorSetLayout destroyed");
    }
}
