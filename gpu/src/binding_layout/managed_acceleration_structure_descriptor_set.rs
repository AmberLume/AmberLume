use crate::binding_layout::descriptor_set_manager::GlobalDescriptorSetBindings;
use ash::vk::{
    AccelerationStructureKHR, DescriptorSet, DescriptorType,
    WriteDescriptorSet, WriteDescriptorSetAccelerationStructureKHR,
};
use ash::Device;

#[derive(Clone)]
pub struct ManagedAccelerationStructureDescriptorSet {
    device: Device,

    descriptor_set: DescriptorSet,
    binding: GlobalDescriptorSetBindings,
}

impl ManagedAccelerationStructureDescriptorSet {
    pub fn new(
        device: Device,
        descriptor_set: DescriptorSet,
        binding: GlobalDescriptorSetBindings,
    ) -> Self {
        Self {
            device,

            descriptor_set,
            binding,
        }
    }

    pub fn write(&self, index: u32, acceleration_structure: AccelerationStructureKHR) {
        let handles = [acceleration_structure];

        let mut acceleration_structure_info =
            WriteDescriptorSetAccelerationStructureKHR::default()
                .acceleration_structures(&handles);

        let mut write = WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_binding(self.binding as u32)
            .dst_array_element(index)
            .descriptor_type(DescriptorType::ACCELERATION_STRUCTURE_KHR)
            .push_next(&mut acceleration_structure_info);
        write.descriptor_count = handles.len() as u32;

        unsafe { self.device.update_descriptor_sets(&[write], &[]) };
    }
}
