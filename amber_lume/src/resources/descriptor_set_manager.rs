use ash::vk::{CommandBuffer, DescriptorBindingFlags, DescriptorImageInfo, DescriptorSet, DescriptorSetLayout, DescriptorType, ImageLayout, PipelineBindPoint, PipelineLayout, Sampler, ShaderStageFlags, WriteDescriptorSet};
use crate::limits::renderer_limits::RendererLimits;
use crate::render::factories::descriptor_set_layout::descriptor_set_layout_factory::{DescriptorSetLayoutBindingDescription, DescriptorSetLayoutFactory};
use anyhow::Result;
use ash::Device;
use crate::render::factories::descriptor_set::descriptor_set_factory::DescriptorSetFactory;
use crate::render::factories::image::managed_image::ManagedImage;

#[repr(u32)]
pub enum GlobalDescriptorSetBindings {
    Texture = 0,
    TextureArray = 1,
    Shadow = 2,
    ShadowArray = 3,
}

pub struct DescriptorSetManager {
    device: Device,

    handle: DescriptorSet,
    layout: DescriptorSetLayout,
}

impl DescriptorSetManager {
    pub fn new(
        device: Device,
        layout_factory: &DescriptorSetLayoutFactory,
        set_factory: &DescriptorSetFactory,
        renderer_limits: &RendererLimits,
    ) -> Result<Self> {
        let max_textures = renderer_limits.image_resource_limits.max_texture_descriptors;
        let max_texture_arrays = renderer_limits.image_resource_limits.max_texture_array_descriptors;
        let max_shadows = renderer_limits.image_resource_limits.max_shadow_descriptors;
        let max_shadow_arrays = renderer_limits.image_resource_limits.max_shadow_array_descriptors;
        
        let bindings = [
            DescriptorSetLayoutBindingDescription {
                binding: GlobalDescriptorSetBindings::Texture as u32,
                binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: max_textures,
                stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
            },
            DescriptorSetLayoutBindingDescription {
                binding: GlobalDescriptorSetBindings::TextureArray as u32,
                binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: max_texture_arrays,
                stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
            },
            DescriptorSetLayoutBindingDescription {
                binding: GlobalDescriptorSetBindings::Shadow as u32,
                binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: max_shadows,
                stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
            },
            DescriptorSetLayoutBindingDescription {
                binding: GlobalDescriptorSetBindings::ShadowArray as u32,
                binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: max_shadow_arrays,
                stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
            },
        ];
        
        let layout = layout_factory.create_descriptor_set_layout(
            "global",
            &bindings, 
        )?;
        
        let descriptors_count = bindings.iter().map(|b| b.descriptor_count).sum::<u32>();
        let handle = set_factory.create_descriptor_set(
            "global",
            &[layout],
            &[descriptors_count]
        )?;
        
        Ok(Self {
            device,

            handle,
            layout,
        })
    }

    pub fn write(&self, binding: GlobalDescriptorSetBindings, index: u32, managed_image: &ManagedImage, sampler: Sampler) {
        let image_info = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(managed_image.image_view)
            .sampler(sampler)];

        let write = WriteDescriptorSet::default()
            .dst_set(self.handle)
            .dst_binding(binding as u32)
            .dst_array_element(index)
            .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_info);

        unsafe { self.device.update_descriptor_sets(&[write], &[]) };
    }

    pub fn bind(&self, command_buffer: CommandBuffer, pipeline_layout: PipelineLayout) {
        unsafe {
            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &[self.handle],
                &[],
            );
        }
    }
    
    pub fn layout(&self) -> &DescriptorSetLayout {
        &self.layout
    }
    
    pub fn destroy(self, layout_factory: &DescriptorSetLayoutFactory) {
        layout_factory.destroy_descriptor_set_layout(self.layout);
    }
}
