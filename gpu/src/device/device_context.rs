use crate::device::physical_device_info::PhysicalDeviceInfo;
use crate::queue::queue_families::QueueFamilies;
use crate::queue::queues::Queues;
use crate::device::vulkan_context::VulkanContext;
use anyhow::{Result, anyhow};
use ash::Device;
use ash::khr::{acceleration_structure, deferred_host_operations, ray_query, shader_draw_parameters, swapchain};
use ash::vk::{DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceAccelerationStructureFeaturesKHR, PhysicalDeviceFeatures, PhysicalDeviceRayQueryFeaturesKHR, PhysicalDeviceVulkan11Features, PhysicalDeviceVulkan12Features, PhysicalDeviceVulkan13Features, SurfaceKHR};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use std::mem::ManuallyDrop;
use std::sync::Arc;
use parking_lot::Mutex;
use tracing::info;
use crate::device::texture_format::TextureFormat;
use crate::utils::debug_utils::DebugUtils;

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

        let ray_tracing = physical_device_info.supports_ray_tracing();
        info!("Ray tracing support: {}", ray_tracing);

        let device = Self::create_device(
            &vulkan_context,
            physical_device_info.handle,
            &queue_families,
            ray_tracing,
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

        self.queues.bind_present(&self.debug_utils, family);

        Ok(())
    }

    pub fn detach_present(&self) {
        self.queues.unbind_present();
    }

    fn create_device(
        vulkan_context: &VulkanContext,
        physical_device: PhysicalDevice,
        queue_families: &QueueFamilies,
        ray_tracing: bool,
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

        let mut extensions = vec![
            swapchain::NAME.as_ptr(),
            shader_draw_parameters::NAME.as_ptr(),
        ];
        if ray_tracing {
            extensions.push(acceleration_structure::NAME.as_ptr());
            extensions.push(deferred_host_operations::NAME.as_ptr());
            extensions.push(ray_query::NAME.as_ptr());
        }
        info!("Created device extensions: {:?}", extensions);

        let physical_device_features = PhysicalDeviceFeatures::default()
            .draw_indirect_first_instance(true)
            .fill_mode_non_solid(true)
            .sampler_anisotropy(true)
            .shader_int64(true)
            .shader_storage_image_extended_formats(true)
            .shader_storage_image_write_without_format(true)
            .shader_storage_image_read_without_format(true);

        let mut features_1_1 = PhysicalDeviceVulkan11Features::default()
            .shader_draw_parameters(true)
            .multiview(true);

        let mut features_1_2 = PhysicalDeviceVulkan12Features::default()
            .buffer_device_address(true)
            .draw_indirect_count(true)
            .descriptor_indexing(true)
            .descriptor_binding_sampled_image_update_after_bind(true)
            .descriptor_binding_storage_image_update_after_bind(true)
            .descriptor_binding_partially_bound(true)
            .runtime_descriptor_array(true)
            .descriptor_binding_variable_descriptor_count(true)
            .shader_sampled_image_array_non_uniform_indexing(true)
            .host_query_reset(true);

        let mut features_1_3 = PhysicalDeviceVulkan13Features::default()
            .synchronization2(true)
            .dynamic_rendering(true)
            .maintenance4(true);

        let mut features_acceleration_structure = PhysicalDeviceAccelerationStructureFeaturesKHR::default()
            .acceleration_structure(true);

        let mut features_ray_query = PhysicalDeviceRayQueryFeaturesKHR::default()
            .ray_query(true);

        let mut device_create_info = DeviceCreateInfo::default()
            .queue_create_infos(&device_queue_create_info)
            .enabled_extension_names(&extensions)
            .enabled_features(&physical_device_features)
            .push_next(&mut features_1_1)
            .push_next(&mut features_1_2)
            .push_next(&mut features_1_3);

        if ray_tracing {
            device_create_info = device_create_info
                .push_next(&mut features_acceleration_structure)
                .push_next(&mut features_ray_query);
        }

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
