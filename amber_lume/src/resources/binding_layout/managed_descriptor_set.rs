use crate::resources::binding_layout::descriptor_set_manager::GlobalDescriptorSetBindings;
use ash::vk::{
    DescriptorImageInfo, DescriptorSet, DescriptorType, ImageLayout, ImageView,
    WriteDescriptorSet,
};
use ash::Device;

#[derive(Clone)]
pub struct ManagedDescriptorSet {
    device: Device,

    descriptor_set: DescriptorSet,
    binding: GlobalDescriptorSetBindings,

    descriptor_type: DescriptorType,
}

impl ManagedDescriptorSet {
    pub fn new(
        device: Device,
        descriptor_set: DescriptorSet,
        binding: GlobalDescriptorSetBindings,
        descriptor_type: DescriptorType,
    ) -> Self {
        Self {
            device,

            descriptor_set,
            binding,

            descriptor_type,
        }
    }

    pub fn write(&self, index: u32, image_view: ImageView) {
        let image_info = [DescriptorImageInfo::default()
            .image_layout(self.image_layout())
            .image_view(image_view)];

        let write = WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_binding(self.binding as u32)
            .dst_array_element(index)
            .descriptor_type(self.descriptor_type)
            .image_info(&image_info);

        unsafe { self.device.update_descriptor_sets(&[write], &[]) };
    }

    pub fn fill_with_default(&self, image_view: ImageView, count: u32) {
        if count == 0 {
            return;
        }

        let info = DescriptorImageInfo::default()
            .image_layout(self.image_layout())
            .image_view(image_view);
        let image_info = vec![info; count as usize];

        let write = WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_binding(self.binding as u32)
            .dst_array_element(0)
            .descriptor_type(self.descriptor_type)
            .image_info(&image_info);

        unsafe { self.device.update_descriptor_sets(&[write], &[]) };
    }

    fn image_layout(&self) -> ImageLayout {
        match self.descriptor_type {
            DescriptorType::STORAGE_IMAGE => ImageLayout::GENERAL,
            _ => ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        }
    }
}
