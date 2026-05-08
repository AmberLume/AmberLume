use crate::render::builder::context_profile::ContextProfile;
use crate::render::device::physical_device_info::PhysicalDeviceInfo;
use anyhow::Result;
use ash::khr::surface::Instance as SurfaceLoader;
use ash::vk::{ApplicationInfo, InstanceCreateInfo, make_api_version, ValidationFeaturesEXT};
use ash::{Entry, Instance, vk};
use std::ffi::{CStr, c_char};
use ash::ext::debug_utils;
use tracing::info;
use crate::render::device::layers::VulkanLayer;

pub struct VulkanContext {
    pub entry: Entry,

    pub instance: Instance,
    pub surface_loader: SurfaceLoader,

    pub physical_devices: Vec<PhysicalDeviceInfo>,
}

impl VulkanContext {
    pub fn new(context_profile: ContextProfile) -> Result<Self> {
        let entry = Entry::linked();

        let instance = Self::create_instance(&entry, context_profile)?;
        let surface_loader = Self::create_surface_loader(&entry, &instance)?;

        let physical_devices = PhysicalDeviceInfo::create_all(&instance)?;

        info!("VulkanContext created");

        Ok(Self {
            entry,

            instance,
            surface_loader,

            physical_devices,
        })
    }

    fn create_instance(entry: &Entry, context_profile: ContextProfile) -> Result<Instance> {
        let app_name = CStr::from_bytes_with_nul(b"Lume\0")?;
        let app_version = make_api_version(0, 0, 1, 0);
        let engine_name = CStr::from_bytes_with_nul(b"AmberLume\0")?;
        let engine_version = make_api_version(0, 0, 1, 0);
        let app_info = ApplicationInfo::default()
            .application_name(app_name)
            .application_version(app_version)
            .engine_name(engine_name)
            .engine_version(engine_version)
            .api_version(vk::API_VERSION_1_3);

        let mut extension_names: Vec<*const c_char> = vec![
            debug_utils::NAME.as_ptr(),
        ];
        extension_names.extend(context_profile.extensions);

        let instance_layers = context_profile.layers.iter()
            .map(|layer| layer.as_vulkan_value())
            .collect::<Vec<_>>();

        let validation_features_enable = context_profile.validation_features.iter()
            .map(|feature| feature.as_vulkan_feature())
            .collect::<Vec<_>>();

        let mut validation_features = ValidationFeaturesEXT::default()
            .enabled_validation_features(&validation_features_enable);

        let mut instance_create_info = InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&instance_layers);

        if context_profile.layers.contains(&VulkanLayer::Validation) && !validation_features_enable.is_empty() {
            instance_create_info = instance_create_info.push_next(&mut validation_features);
        }

        let instance = unsafe { entry.create_instance(&instance_create_info, None)? };

        Ok(instance)
    }

    fn create_surface_loader(entry: &Entry, instance: &Instance) -> Result<SurfaceLoader> {
        let surface_loader = SurfaceLoader::new(&entry, &instance);

        Ok(surface_loader)
    }

    pub fn destroy(&self) -> Result<()> {
        unsafe { self.instance.destroy_instance(None) };

        info!("VulkanContext destroyed");

        Ok(())
    }
}
