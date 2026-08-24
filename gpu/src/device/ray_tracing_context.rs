use crate::device::device_context::DeviceContext;
use crate::device::ray_tracing_properties::RayTracingProperties;
use crate::device::vulkan_context::VulkanContext;
use ash::khr::acceleration_structure::Device as AccelerationStructureDevice;
use tracing::info;

#[derive(Clone)]
pub struct RayTracingContext {
    pub device: AccelerationStructureDevice,
    pub properties: RayTracingProperties,
}

impl RayTracingContext {
    pub fn new(vulkan_context: &VulkanContext, device_context: &DeviceContext) -> Self {
        let device = AccelerationStructureDevice::new(&vulkan_context.instance, &device_context.device);
        let properties = RayTracingProperties::query(vulkan_context, device_context);

        info!("RayTracingContext created");

        Self { device, properties }
    }
}
