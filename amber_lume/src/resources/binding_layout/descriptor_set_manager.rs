use ash::vk::{CommandBuffer, CopyDescriptorSet, DescriptorBindingFlags, DescriptorImageInfo, DescriptorSet, DescriptorSetLayout, DescriptorType, ImageLayout, ImageView, PipelineBindPoint, PipelineLayout, ShaderStageFlags, WriteDescriptorSet};
use crate::render::factories::descriptor_set_layout::descriptor_set_layout_factory::{DescriptorSetLayoutBindingDescription, DescriptorSetLayoutFactory};
use anyhow::Result;
use ash::Device;
use tracing::warn;
use crate::limits::ResourceLimits;
use crate::render::factories::descriptor_set::descriptor_set_factory::DescriptorSetFactory;
use crate::render::factories::image::managed_image::ManagedImage;
use crate::render::factories::sampler::sampler_factory::SamplerFactory;
use crate::resources::sampler_registry::{SamplerRegistry, SamplerType};

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GlobalDescriptorSetBindings {
    Texture = 0,
    ShadowArray = 1,
    StorageImage = 2,
}

pub struct DescriptorSetManager {
    device: Device,

    handle: DescriptorSet,
    layout: DescriptorSetLayout,

    sampler_registry: SamplerRegistry,
}

impl DescriptorSetManager {
    pub fn new(
        device: Device,
        layout_factory: &DescriptorSetLayoutFactory,
        set_factory: &DescriptorSetFactory,
        sampler_factory: &SamplerFactory,
        limits: &ResourceLimits,
    ) -> Result<Self> {
        let max_textures = limits.max_texture_descriptors;
        let max_shadow_arrays = limits.max_shadow_array_descriptors;
        let max_storage_images = limits.max_storage_image_descriptors;

        let bindings = [
            DescriptorSetLayoutBindingDescription {
                binding: GlobalDescriptorSetBindings::Texture,
                binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: max_textures,
                stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
            },
            DescriptorSetLayoutBindingDescription {
                binding: GlobalDescriptorSetBindings::ShadowArray,
                binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: max_shadow_arrays,
                stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
            },
            DescriptorSetLayoutBindingDescription {
                binding: GlobalDescriptorSetBindings::StorageImage,
                binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                descriptor_type: DescriptorType::STORAGE_IMAGE,
                descriptor_count: max_storage_images,
                stage_flags: ShaderStageFlags::COMPUTE,
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
        
        let sampler_registry = SamplerRegistry::create(&sampler_factory)?;

        Ok(Self {
            device,

            handle,
            layout,

            sampler_registry,
        })
    }

    pub fn write(
        &self,
        binding: GlobalDescriptorSetBindings,
        index: u32,
        managed_image: &ManagedImage,
        sampler: SamplerType,
    ) {
        let sampler = self.sampler_registry.get(sampler);
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

    pub fn write_storage(
        &self,
        index: u32,
        image_view: ImageView,
    ) {
        let image_info = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::GENERAL)
            .image_view(image_view)];

        let write = WriteDescriptorSet::default()
            .dst_set(self.handle)
            .dst_binding(GlobalDescriptorSetBindings::StorageImage as u32)
            .dst_array_element(index)
            .descriptor_type(DescriptorType::STORAGE_IMAGE)
            .image_info(&image_info);

        unsafe { self.device.update_descriptor_sets(&[write], &[]) };
    }

    pub fn fill_binding_with_default(
        &self,
        binding: GlobalDescriptorSetBindings,
        managed_image: &ManagedImage,
        sampler: SamplerType,
        count: u32,
    ) {
        if count == 0 {
            return;
        }

        let sampler = self.sampler_registry.get(sampler);
        let info = DescriptorImageInfo::default()
            .image_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(managed_image.image_view)
            .sampler(sampler);
        let image_info = vec![info; count as usize];

        let write = WriteDescriptorSet::default()
            .dst_set(self.handle)
            .dst_binding(binding as u32)
            .dst_array_element(0)
            .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_info);

        unsafe { self.device.update_descriptor_sets(&[write], &[]) };
    }

    pub fn copy(
        &self,
        binding: GlobalDescriptorSetBindings,
        src_index: u32,
        dst_index: u32,
    ) {
        if src_index == dst_index {
            warn!("Trying to copy same descriptor index");

            return;
        }

        let binding_index = binding as u32;

        let copy = CopyDescriptorSet::default()
            .src_set(self.handle)
            .src_binding(binding_index)
            .src_array_element(src_index)
            .dst_set(self.handle)
            .dst_binding(binding_index)
            .dst_array_element(dst_index)
            .descriptor_count(1);

        unsafe { self.device.update_descriptor_sets(&[], &[copy]) };
    }

    pub fn bind(&self, command_buffer: CommandBuffer, pipeline_layout: PipelineLayout) {
        for bind_point in [PipelineBindPoint::GRAPHICS, PipelineBindPoint::COMPUTE] {
            unsafe {
                self.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    bind_point,
                    pipeline_layout,
                    0,
                    &[self.handle],
                    &[],
                );
            }
        }
    }

    pub fn layout(&self) -> &DescriptorSetLayout {
        &self.layout
    }
    
    pub fn destroy(
        self,
        sampler_factory: &SamplerFactory,
        descriptor_set_layout_factory: &DescriptorSetLayoutFactory,
    ) {
        self.sampler_registry.destroy(&sampler_factory);

        descriptor_set_layout_factory.destroy_descriptor_set_layout(self.layout);
    }
}
