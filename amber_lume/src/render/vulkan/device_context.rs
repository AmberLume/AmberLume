use crate::render::vulkan::physical_device_info::PhysicalDeviceInfo;
use crate::render::vulkan::queue::queue_families::QueueFamilies;
use crate::render::vulkan::queue::queues::Queues;
use crate::render::vulkan::surface::vulkan_surface::VulkanSurface;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::{Result, anyhow};
use ash::Device;
use ash::khr::{dynamic_rendering, shader_draw_parameters, swapchain};
use ash::vk::{DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceDynamicRenderingFeaturesKHR, PhysicalDeviceFeatures, PhysicalDeviceVulkan12Features};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};
use tracing::info;
use crate::render::vulkan::debug_utils::DebugUtils;

pub struct DeviceContext {
    pub device: Device,
    pub physical_device_info: PhysicalDeviceInfo,
    pub debug_utils: Arc<DebugUtils>,

    pub queues: Arc<Queues>,

    pub allocator: Arc<Mutex<ManuallyDrop<Allocator>>>,
}

impl DeviceContext {
    pub fn new(vulkan_context: &VulkanContext, vulkan_surface: &VulkanSurface) -> Result<Self> {
        let physical_device_info = vulkan_context
            .physical_devices
            .iter()
            .find(|physical_device| {
                physical_device
                    .is_suitable_for(&vulkan_context, &vulkan_surface)
                    .is_ok()
            })
            .cloned()
            .ok_or_else(|| anyhow!("No suitable device found"))?;

        let queue_families = QueueFamilies::find(
            &vulkan_context,
            &vulkan_surface,
            physical_device_info.handle,
        )?;

        let device = Self::create_device(
            &vulkan_context,
            physical_device_info.handle,
            &queue_families,
        )?;

        let debug_utils = DebugUtils::create(&vulkan_context, &device);
        
        let queues = Queues::new(&device, &debug_utils, &queue_families);

        let allocator =
            Self::create_allocator(&vulkan_context, &device, &physical_device_info.handle)?;

        info!("DeviceContext created");

        Ok(Self {
            device,
            physical_device_info,
            debug_utils: Arc::new(debug_utils),

            queues: Arc::new(queues),

            allocator: Arc::new(Mutex::new(ManuallyDrop::new(allocator))),
        })
    }

    fn create_device(
        vulkan_context: &VulkanContext,
        physical_device: PhysicalDevice,
        queue_families: &QueueFamilies,
    ) -> Result<Device> {
        let unique = queue_families.unique_families();
        let priorities = [1.0f32];
        let device_queue_create_info: Vec<_> = unique
            .iter()
            .map(|&i| {
                DeviceQueueCreateInfo::default()
                    .queue_family_index(i)
                    .queue_priorities(&priorities)
            })
            .collect();

        let extensions = [
            swapchain::NAME.as_ptr(),
            dynamic_rendering::NAME.as_ptr(),
            shader_draw_parameters::NAME.as_ptr(),
        ];
        info!("Created device extensions: {:?}", extensions);

        let physical_device_features = PhysicalDeviceFeatures::default()
            .sampler_anisotropy(true)
            .shader_int64(true);

        let mut features_1_2 = PhysicalDeviceVulkan12Features::default()
            .buffer_device_address(true)
            .draw_indirect_count(true)
            .descriptor_indexing(true)
            .descriptor_binding_sampled_image_update_after_bind(true)
            .descriptor_binding_partially_bound(true)
            .runtime_descriptor_array(true)
            .descriptor_binding_variable_descriptor_count(true)
            .shader_sampled_image_array_non_uniform_indexing(true);

        let mut dynamic_rendering_features = PhysicalDeviceDynamicRenderingFeaturesKHR::default()
            .dynamic_rendering(true);

        let device_create_info = DeviceCreateInfo::default()
            .queue_create_infos(&device_queue_create_info)
            .enabled_extension_names(&extensions)
            .enabled_features(&physical_device_features)
            .push_next(&mut features_1_2)
            .push_next(&mut dynamic_rendering_features);

        let device = unsafe {
            vulkan_context
                .instance
                .create_device(physical_device, &device_create_info, None)?
        };

        info!("Logical device created");

        Ok(device)
    }

    fn create_allocator(
        vulkan_context: &VulkanContext,
        device: &Device,
        physical_device: &PhysicalDevice,
    ) -> Result<Allocator> {
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: vulkan_context.instance.clone(),
            device: device.clone(),
            physical_device: physical_device.clone(),
            debug_settings: Default::default(),
            buffer_device_address: true,
            allocation_sizes: Default::default(),
        })?;

        info!("Memory allocator created");

        Ok(allocator)
    }

    pub fn destroy(&mut self) -> Result<()> {
        self.queues.all_wait_idle()?;

        let mut allocator = self.allocator.lock().unwrap();
        unsafe { ManuallyDrop::drop(&mut *allocator) };

        unsafe { self.device.destroy_device(None) };

        info!("DeviceContext destroyed");

        Ok(())
    }
}
