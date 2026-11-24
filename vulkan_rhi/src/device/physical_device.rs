use crate::device::device_features::DeviceFeatures;
use anyhow::{Result, anyhow, bail};
use ash::Instance;
use ash::khr::swapchain;
use ash::vk::{self, API_VERSION_1_3, PhysicalDeviceProperties, PhysicalDeviceType, QueueFlags};
use logger::info;
use std::ffi::CStr;

pub(crate) struct PhysicalDevice {
    pub handle: vk::PhysicalDevice,
    pub name: String,

    pub graphics_family: u32,
}

impl PhysicalDevice {
    pub const EXTENSIONS: [&'static CStr; 1] = [swapchain::NAME];

    pub fn select(instance: &Instance) -> Result<Self> {
        let handles = unsafe { instance.enumerate_physical_devices()? };

        let mut selected: Option<(Self, bool)> = None;

        for handle in handles {
            let properties = unsafe { instance.get_physical_device_properties(handle) };
            let name = properties
                .device_name_as_c_str()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            let discrete = properties.device_type == PhysicalDeviceType::DISCRETE_GPU;

            match Self::check(instance, handle, &properties) {
                Ok(graphics_family) => {
                    info!("Physical device {} is suitable, discrete {}", name, discrete);

                    let better = match &selected {
                        Some((_, selected_discrete)) => discrete && !selected_discrete,
                        None => true,
                    };

                    if better {
                        selected = Some((Self { handle, name, graphics_family }, discrete));
                    }
                }
                Err(reason) => info!("Physical device {} is not suitable: {}", name, reason),
            }
        }

        let (physical_device, _) = selected.ok_or_else(|| anyhow!("No suitable physical device"))?;

        info!("Physical device selected: {}", physical_device.name);

        Ok(physical_device)
    }

    fn check(instance: &Instance, handle: vk::PhysicalDevice, properties: &PhysicalDeviceProperties) -> Result<u32> {
        if properties.api_version < API_VERSION_1_3 {
            bail!("Vulkan 1.3 is not supported");
        }

        let extensions = unsafe { instance.enumerate_device_extension_properties(handle)? };

        for name in Self::EXTENSIONS {
            if !extensions.iter().any(|extension| extension.extension_name_as_c_str() == Ok(name)) {
                bail!("{:?} is not supported", name);
            }
        }

        let missing = DeviceFeatures::query(instance, handle).missing(&DeviceFeatures::REQUIRED);

        if !missing.is_empty() {
            bail!("features not supported: {:?}", missing);
        }

        let families = unsafe { instance.get_physical_device_queue_family_properties(handle) };

        families
            .iter()
            .position(|family| family.queue_flags.contains(QueueFlags::GRAPHICS))
            .map(|index| index as u32)
            .ok_or_else(|| anyhow!("No graphics queue family"))
    }
}
