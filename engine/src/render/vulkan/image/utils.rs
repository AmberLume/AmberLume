use anyhow::bail;
use ash::Instance;
use ash::vk::{MemoryPropertyFlags, PhysicalDevice};

pub fn find_memory_type_index(
    instance: &Instance,
    physical_device: PhysicalDevice,
    type_bits: u32,
    properties: MemoryPropertyFlags,
) -> anyhow::Result<u32> {
    let physical_device_memory_properties =
        unsafe { instance.get_physical_device_memory_properties(physical_device) };

    for index in 0..physical_device_memory_properties.memory_type_count {
        let contains_properties = physical_device_memory_properties.memory_types[index as usize]
            .property_flags
            .contains(properties);
        if (type_bits & (1 << index)) != 0 && contains_properties {
            return Ok(index);
        }
    }

    bail!("No such memory type");
}
