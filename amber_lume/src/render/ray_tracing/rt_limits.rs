use ash::vk::{PhysicalDeviceAccelerationStructurePropertiesKHR, PhysicalDeviceProperties2};
use tracing::info;
use gpu::DeviceContext;
use gpu::VulkanContext;

#[derive(Clone, Copy, Debug)]
pub struct RTLimits {
    pub min_scratch_offset_alignment: u32,
}

impl RTLimits {
    pub fn query(
        vulkan_context: &VulkanContext,
        device_context: &DeviceContext,
    ) -> Self {
        let mut as_properties = PhysicalDeviceAccelerationStructurePropertiesKHR::default();
        let mut properties = PhysicalDeviceProperties2::default()
            .push_next(&mut as_properties);

        unsafe {
            vulkan_context
                .instance
                .get_physical_device_properties2(
                    device_context.physical_device_info.handle,
                    &mut properties,
                )
        };

        let rt_limits = Self {
            min_scratch_offset_alignment: as_properties.min_acceleration_structure_scratch_offset_alignment,
        };

        info!("RTLimits: {:?}", rt_limits);

        rt_limits
    }
}
