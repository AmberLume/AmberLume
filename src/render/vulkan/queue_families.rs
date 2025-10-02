use crate::render::vulkan::physical_device_info::PhysicalDeviceInfo;
use crate::render::vulkan::vk_context::VkContext;
use crate::render::vulkan::vk_surface::VkSurface;
use anyhow::Result;
use ash::vk;
use tracing::{info, instrument};
use vk::QueueFlags;

#[derive(Clone, Copy, Debug)]
pub struct QueueFamilies {
    pub graphics: u32,
    pub present: u32,
}

impl QueueFamilies {
    #[instrument(level = "trace", skip_all)]
    pub fn find(
        vk_context: &VkContext,
        vk_surface: &VkSurface,
        physical_device_info: &PhysicalDeviceInfo,
    ) -> Result<Self> {
        let queue_family_properties = unsafe {
            vk_context
                .instance
                .get_physical_device_queue_family_properties(physical_device_info.handle)
        };
        let mut graphics = None;
        let mut present = None;

        info!("Searching for graphics queue families");
        for (i, properties) in queue_family_properties.iter().enumerate() {
            if properties.queue_flags.contains(QueueFlags::GRAPHICS) {
                graphics = Some(i);
            }
            let ok = unsafe {
                vk_context
                    .surface_loader
                    .get_physical_device_surface_support(
                        physical_device_info.handle,
                        i as u32,
                        vk_surface.surface,
                    )?
            };
            if ok {
                present = Some(i);
            }
            if graphics.is_some() && present.is_some() {
                break;
            }
        }

        info!("Found graphics queue families: {:?}", graphics);

        Ok(Self {
            graphics: graphics.unwrap() as u32,
            present: present.unwrap() as u32,
        })
    }
}
