use anyhow::Result;
use ash::Device;
use ash::vk::{
    DescriptorBindingFlags, DescriptorSetLayout, DescriptorSetLayoutBinding,
    DescriptorSetLayoutBindingFlagsCreateInfo, DescriptorSetLayoutCreateFlags,
    DescriptorSetLayoutCreateInfo, DescriptorType, PipelineLayout, PipelineLayoutCreateInfo,
    PushConstantRange, ShaderStageFlags,
};
use std::slice;

#[repr(C)]
pub struct PushConstants {
    pub vertex_buffer_address: u64,
    pub index_buffer_address: u64,
    pub entity_buffer_address: u64,
    pub material_buffer_address: u64,
}

pub struct GraphicsPipelineLayout {
    pub handle: PipelineLayout,
}

impl GraphicsPipelineLayout {
    pub fn create(device: &Device) -> Result<Self> {
        let per_frame_set_layout = Self::create_per_frame_set_layout(&device)?;
        let bindless_set_layout = Self::create_bindless_set_layout(&device)?;

        let set_layouts = [per_frame_set_layout, bindless_set_layout];

        let push_range = PushConstantRange {
            stage_flags: ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
            offset: 0,
            size: size_of::<PushConstants>() as u32,
        };

        let layout_info = PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts)
            .push_constant_ranges(slice::from_ref(&push_range));

        let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None)? };

        Ok(Self {
            handle: pipeline_layout,
        })
    }

    pub fn create_per_frame_set_layout(device: &Device) -> Result<DescriptorSetLayout> {
        let bindings = [
            // Camera
            DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT),
        ];

        let layout_info = DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

        let descriptor_set_layout =
            unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        Ok(descriptor_set_layout)
    }

    pub fn create_bindless_set_layout(device: &Device) -> Result<DescriptorSetLayout> {
        let bindings = [
            // Textures
            DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(10000)
                .stage_flags(ShaderStageFlags::FRAGMENT),
            // Samplers
            DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(DescriptorType::SAMPLER)
                .descriptor_count(100)
                .stage_flags(ShaderStageFlags::FRAGMENT),
        ];

        let binding_flags = [
            DescriptorBindingFlags::PARTIALLY_BOUND | DescriptorBindingFlags::UPDATE_AFTER_BIND,
            DescriptorBindingFlags::PARTIALLY_BOUND | DescriptorBindingFlags::UPDATE_AFTER_BIND,
        ];

        let mut binding_flags_info =
            DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&binding_flags);

        let layout_info = DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings)
            .flags(DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .push_next(&mut binding_flags_info);

        let layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        Ok(layout)
    }
}
