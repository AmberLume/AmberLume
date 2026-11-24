use ash::Instance;
use ash::vk::{Bool32, FALSE, PhysicalDevice, PhysicalDeviceFeatures2, PhysicalDeviceVulkan13Features};

pub(crate) struct DeviceFeatures {
    pub dynamic_rendering: bool,
    pub synchronization2: bool,
}

impl DeviceFeatures {
    pub const REQUIRED: Self = Self {
        dynamic_rendering: true,
        synchronization2: true,
    };

    pub fn query(instance: &Instance, physical_device: PhysicalDevice) -> Self {
        let mut features_1_3 = PhysicalDeviceVulkan13Features::default();
        let mut features = PhysicalDeviceFeatures2::default().push_next(&mut features_1_3);

        unsafe { instance.get_physical_device_features2(physical_device, &mut features) };

        Self {
            dynamic_rendering: Self::supported(features_1_3.dynamic_rendering),
            synchronization2: Self::supported(features_1_3.synchronization2),
        }
    }

    pub fn missing(&self, required: &Self) -> Vec<&'static str> {
        [
            ("dynamicRendering", self.dynamic_rendering, required.dynamic_rendering),
            ("synchronization2", self.synchronization2, required.synchronization2),
        ]
        .into_iter()
        .filter(|(_, supported, required)| *required && !*supported)
        .map(|(name, _, _)| name)
        .collect()
    }

    pub fn vulkan_1_3(&self) -> PhysicalDeviceVulkan13Features<'static> {
        PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(self.dynamic_rendering)
            .synchronization2(self.synchronization2)
    }

    fn supported(value: Bool32) -> bool {
        value != FALSE
    }
}
