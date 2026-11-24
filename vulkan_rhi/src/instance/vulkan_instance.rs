use crate::instance::debug_messenger::DebugMessenger;
use crate::instance::validation_layer::ValidationLayer;
use anyhow::Result;
use ash::ext::{debug_utils, metal_surface, swapchain_colorspace};
use ash::khr::{android_surface, surface, wayland_surface, win32_surface, xcb_surface, xlib_surface};
use ash::vk::{API_VERSION_1_3, ApplicationInfo, ExtensionProperties, InstanceCreateInfo, ValidationFeaturesEXT, make_api_version};
use ash::{Entry, Instance};
use explicit_rhi::Validation;
use logger::info;
use std::ffi::{CStr, c_char};

pub(crate) struct VulkanInstance {
    pub instance: Instance,

    pub debug_messenger: Option<DebugMessenger>,
}

impl VulkanInstance {
    const EXTENSIONS: [&'static CStr; 9] = [
        surface::NAME,
        wayland_surface::NAME,
        xlib_surface::NAME,
        xcb_surface::NAME,
        win32_surface::NAME,
        android_surface::NAME,
        metal_surface::NAME,
        swapchain_colorspace::NAME,
        debug_utils::NAME,
    ];

    pub fn create(validation: Validation) -> Result<Self> {
        let entry = Entry::linked();

        let available = unsafe { entry.enumerate_instance_extension_properties(None)? };
        let extensions = Self::EXTENSIONS
            .iter()
            .filter(|name| Self::is_available(&available, name))
            .map(|name| name.as_ptr())
            .collect::<Vec<*const c_char>>();

        let validation_layer = ValidationLayer::create(&entry, validation)?;
        let layers = validation_layer
            .iter()
            .map(|_| ValidationLayer::NAME.as_ptr())
            .collect::<Vec<*const c_char>>();

        let application_info = ApplicationInfo::default()
            .application_name(c"Lume")
            .application_version(make_api_version(0, 0, 1, 0))
            .engine_name(c"AmberLume")
            .engine_version(make_api_version(0, 0, 1, 0))
            .api_version(API_VERSION_1_3);

        let mut validation_features = ValidationFeaturesEXT::default();

        let mut create_info = InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layers);

        if let Some(validation_layer) = &validation_layer
            && !validation_layer.features.is_empty()
        {
            validation_features = validation_features.enabled_validation_features(&validation_layer.features);
            create_info = create_info.push_next(&mut validation_features);
        }

        let instance = unsafe { entry.create_instance(&create_info, None)? };

        let debug_messenger = match validation_layer {
            Some(_) if Self::is_available(&available, debug_utils::NAME) => Some(DebugMessenger::create(&entry, &instance)?),
            _ => None,
        };

        info!(
            "VulkanInstance created: validation {}, features {:?}",
            debug_messenger.is_some(),
            validation_layer.map(|layer| layer.features).unwrap_or_default(),
        );

        Ok(Self {
            instance,

            debug_messenger,
        })
    }

    pub fn destroy(&self) {
        if let Some(debug_messenger) = &self.debug_messenger {
            debug_messenger.destroy();
        }

        unsafe { self.instance.destroy_instance(None) };

        info!("VulkanInstance destroyed");
    }

    fn is_available(available: &[ExtensionProperties], name: &CStr) -> bool {
        available.iter().any(|extension| extension.extension_name_as_c_str() == Ok(name))
    }
}
