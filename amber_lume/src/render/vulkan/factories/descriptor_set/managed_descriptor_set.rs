use ash::Device;
use ash::vk::{CommandBuffer, DescriptorImageInfo, DescriptorSet, DescriptorType, ImageLayout, PipelineBindPoint, PipelineLayout, Sampler, WriteDescriptorSet};
use crate::render::vulkan::factories::image::managed_image::ManagedImage;

pub struct ManagedDescriptorSet {
    device: Device,
    
    handle: DescriptorSet,
}

impl ManagedDescriptorSet {
    pub fn create(
        device: Device,
        handle: DescriptorSet,
    ) -> Self {
        Self {
            device,
            handle,
        }
    }
    
    pub fn bind_image(
        &self,
        array_index: u32,
        binding: u32,
        managed_image: &ManagedImage,
        sampler: Sampler,
    ) {
        let image_info = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(managed_image.image_view)
            .sampler(sampler)];

        let write = WriteDescriptorSet::default()
            .dst_set(self.handle)
            .dst_binding(binding)
            .dst_array_element(array_index)
            .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_info);

        unsafe { self.device.update_descriptor_sets(&[write], &[]) };
    }
    
    pub fn bind(
        &self,
        command_buffer: CommandBuffer,
        pipeline_layout: PipelineLayout,
    ) {
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
}