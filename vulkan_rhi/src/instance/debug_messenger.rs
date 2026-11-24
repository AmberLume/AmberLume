use anyhow::Result;
use ash::ext::debug_utils;
use ash::vk::{
    Bool32, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
    DebugUtilsMessengerCallbackDataEXT, DebugUtilsMessengerCreateInfoEXT, DebugUtilsMessengerEXT, FALSE,
};
use ash::{Entry, Instance};
use logger::{error, warn};
use std::ffi::{CStr, c_void};

pub(crate) struct DebugMessenger {
    loader: debug_utils::Instance,
    messenger: DebugUtilsMessengerEXT,
}

impl DebugMessenger {
    pub fn create(entry: &Entry, instance: &Instance) -> Result<Self> {
        let loader = debug_utils::Instance::new(entry, instance);

        let create_info = DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(DebugUtilsMessageSeverityFlagsEXT::ERROR | DebugUtilsMessageSeverityFlagsEXT::WARNING)
            .message_type(
                DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(Self::callback));

        let messenger = unsafe { loader.create_debug_utils_messenger(&create_info, None)? };

        Ok(Self { loader, messenger })
    }

    pub fn destroy(self) {
        unsafe { self.loader.destroy_debug_utils_messenger(self.messenger, None) };
    }

    unsafe extern "system" fn callback(
        severity: DebugUtilsMessageSeverityFlagsEXT,
        message_type: DebugUtilsMessageTypeFlagsEXT,
        data: *const DebugUtilsMessengerCallbackDataEXT<'_>,
        _user_data: *mut c_void,
    ) -> Bool32 {
        let message = if data.is_null() || unsafe { (*data).p_message.is_null() } {
            "".into()
        } else {
            unsafe { CStr::from_ptr((*data).p_message) }.to_string_lossy()
        };

        if severity.contains(DebugUtilsMessageSeverityFlagsEXT::ERROR) {
            error!("{:?}: {}", message_type, message);
        } else {
            warn!("{:?}: {}", message_type, message);
        }

        FALSE
    }
}
