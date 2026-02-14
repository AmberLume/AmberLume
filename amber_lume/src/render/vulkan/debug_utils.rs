use std::ffi::CString;
use ash::ext::debug_utils::Device;
use ash::vk::{DebugUtilsObjectNameInfoEXT, Handle};
use crate::render::vulkan::vulkan_context::VulkanContext;

pub struct DebugUtils {
    device: Device,
}

impl DebugUtils {
    pub fn create(vulkan_context: &VulkanContext, device: &ash::Device) -> Self {
        let device = Device::new(&vulkan_context.instance, &device);
        
        Self {
            device, 
        }
    }

    pub fn label<T: Handle>(
        &self,
        object: T,
        name: &str,
    ) {
        let c_name = CString::new(name).unwrap();
        let name_info = DebugUtilsObjectNameInfoEXT::default()
            .object_handle(object)
            .object_name(&c_name);

        unsafe { let _ = self.device.set_debug_utils_object_name(&name_info); };
    }
}
