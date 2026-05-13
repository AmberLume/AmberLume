use std::ffi::c_char;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum VulkanLayer {
    Validation,
}

impl VulkanLayer {
    pub fn as_vulkan_value(&self) -> *const c_char {
        let raw = match self {
            VulkanLayer::Validation => b"VK_LAYER_KHRONOS_validation\0",
        };

        raw.as_ptr() as *const c_char
    }
}
