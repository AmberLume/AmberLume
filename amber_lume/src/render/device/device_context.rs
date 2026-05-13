use crate::render::device::physical_device_info::PhysicalDeviceInfo;
use crate::render::queue::queue_families::QueueFamilies;
use crate::render::queue::queues::Queues;
use crate::render::device::vulkan_context::VulkanContext;
use anyhow::{Result, anyhow};
use ash::Device;
use ash::khr::{shader_draw_parameters, swapchain};
use ash::vk::{DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceVulkan11Features, PhysicalDeviceVulkan12Features, PhysicalDeviceVulkan13Features, SurfaceKHR};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use std::mem::ManuallyDrop;
use std::sync::Arc;
use parking_lot::Mutex;
use tracing::info;
use crate::render::device::texture_format::TextureFormat;
use crate::render::utils::debug_utils::DebugUtils;

pub struct DeviceContext {
    pub device: Device,
    pub physical_device_info: PhysicalDeviceInfo,
    pub debug_utils: Arc<DebugUtils>,

    pub queues: Arc<Queues>,
    pub texture_format: TextureFormat,

    pub allocator: Arc<Mutex<ManuallyDrop<Allocator>>>,
}

impl DeviceContext {
    pub fn new(vulkan_context: &VulkanContext) -> Result<Self> {
        let physical_device_info = vulkan_context
            .physical_devices
            .iter()
            .find(|physical_device| physical_device.is_suitable().is_ok())
            .cloned()
            .ok_or_else(|| anyhow!("No suitable device found"))?;

        let queue_families = QueueFamilies::find(
            &vulkan_context,
            physical_device_info.handle,
        )?;

        let device = Self::create_device(
            &vulkan_context,
            physical_device_info.handle,
            &queue_families,
        )?;

        let debug_utils = DebugUtils::create(&vulkan_context, &device);

        let queues = Queues::new(&device, &debug_utils, &queue_families);
        let texture_format = TextureFormat::pick_for_device(&physical_device_info.features);

        let allocator = Self::create_allocator(&vulkan_context, &device, &physical_device_info.handle)?;

        info!("DeviceContext created");

        Ok(Self {
            device,
            physical_device_info,
            debug_utils: Arc::new(debug_utils),

            queues: Arc::new(queues),
            texture_format,

            allocator: Arc::new(Mutex::new(ManuallyDrop::new(allocator))),
        })
    }

    pub fn attach_present(&self, vulkan_context: &VulkanContext, surface: SurfaceKHR) -> Result<()> {
        let family = QueueFamilies::find_present(
            vulkan_context,
            self.physical_device_info.handle,
            surface,
        )?;

        let queue = Queues::create_single_queue(&self.device, &self.debug_utils, family, "present");

        self.queues.bind_present(queue);

        Ok(())
    }

    pub fn detach_present(&self) {
        self.queues.unbind_present();
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
            shader_draw_parameters::NAME.as_ptr(),
        ];
        info!("Created device extensions: {:?}", extensions);

        let physical_device_features = PhysicalDeviceFeatures::default()
            .draw_indirect_first_instance(true)
            .fill_mode_non_solid(true)
            .sampler_anisotropy(true)
            .shader_int64(true);

        let mut features_1_1 = PhysicalDeviceVulkan11Features::default()
            .shader_draw_parameters(true)
            .multiview(true);

        let mut features_1_2 = PhysicalDeviceVulkan12Features::default()
            .buffer_device_address(true)
            .draw_indirect_count(true)
            .descriptor_indexing(true)
            .descriptor_binding_sampled_image_update_after_bind(true)
            .descriptor_binding_partially_bound(true)
            .runtime_descriptor_array(true)
            .descriptor_binding_variable_descriptor_count(true)
            .shader_sampled_image_array_non_uniform_indexing(true)
            .host_query_reset(true);

        let mut features_1_3 = PhysicalDeviceVulkan13Features::default()
            .synchronization2(true)
            .dynamic_rendering(true)
            .maintenance4(true);

        let device_create_info = DeviceCreateInfo::default()
            .queue_create_infos(&device_queue_create_info)
            .enabled_extension_names(&extensions)
            .enabled_features(&physical_device_features)
            .push_next(&mut features_1_1)
            .push_next(&mut features_1_2)
            .push_next(&mut features_1_3);

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

        unsafe { ManuallyDrop::drop(&mut *self.allocator.lock()) };

        unsafe { self.device.destroy_device(None) };

        info!("DeviceContext destroyed");

        Ok(())
    }
}
