use crate::instance::debug_messenger::DebugMessenger;
use crate::instance::validation_layer::ValidationLayer;
use anyhow::{Result, bail};
use ash::ext::{debug_utils, metal_surface, swapchain_colorspace};
use ash::khr::{android_surface, surface, wayland_surface, win32_surface, xcb_surface, xlib_surface};
use ash::vk::{API_VERSION_1_3, ApplicationInfo, ExtensionProperties, InstanceCreateInfo, ValidationFeaturesEXT, make_api_version};
use ash::{Entry, Instance};
use ash_window::enumerate_required_extensions;
use explicit_rhi::{Validation, Window};
use logger::info;
use std::ffi::{CStr, c_char};

pub(crate) struct VulkanInstance {
    pub entry: Entry,
    pub instance: Instance,
    pub surface_loader: surface::Instance,
    pub extensions: Vec<&'static CStr>,

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
            .into_iter()
            .filter(|name| Self::is_available(&available, name))
            .collect::<Vec<&'static CStr>>();
        let extension_names = extensions
            .iter()
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
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layers);

        if let Some(validation_layer) = &validation_layer
            && !validation_layer.features.is_empty()
        {
            validation_features = validation_features.enabled_validation_features(&validation_layer.features);
            create_info = create_info.push_next(&mut validation_features);
        }

        let instance = unsafe { entry.create_instance(&create_info, None)? };

        let validation_enabled = validation_layer.is_some();
        let validation_features = validation_layer.map(|layer| layer.features).unwrap_or_default();

        let debug_messenger = if validation_enabled && extensions.contains(&debug_utils::NAME) {
            match DebugMessenger::create(&entry, &instance) {
                Ok(debug_messenger) => Some(debug_messenger),
                Err(error) => {
                    unsafe { instance.destroy_instance(None) };

                    return Err(error);
                }
            }
        } else {
            None
        };

        info!(
            "VulkanInstance created: validation layer {}, features {:?}, debug messenger {}",
            validation_enabled,
            validation_features,
            debug_messenger.is_some(),
        );

        let surface_loader = surface::Instance::new(&entry, &instance);

        Ok(Self {
            entry,
            instance,
            surface_loader,
            extensions,

            debug_messenger,
        })
    }

    pub fn check_surface(&self, window: &Window) -> Result<()> {
        let required = enumerate_required_extensions(window.display)?;

        for &name in required {
            let name = unsafe { CStr::from_ptr(name) };

            if !self.extensions.contains(&name) {
                bail!("{:?} is not available, the window cannot be shown", name);
            }
        }

        Ok(())
    }

    pub fn destroy(self) {
        if let Some(debug_messenger) = self.debug_messenger {
            debug_messenger.destroy();
        }

        unsafe { self.instance.destroy_instance(None) };

        info!("VulkanInstance destroyed");
    }

    fn is_available(available: &[ExtensionProperties], name: &CStr) -> bool {
        available.iter().any(|extension| extension.extension_name_as_c_str() == Ok(name))
    }
}
