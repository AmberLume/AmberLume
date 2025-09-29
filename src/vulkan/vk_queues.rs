use anyhow::Result;
use ash::vk::{PhysicalDevice, QueueFlags, SurfaceKHR};
use ash::Instance;
use tracing::instrument;

#[derive(Clone, Copy, Debug)]
pub struct VkQueues {
    pub graphics_family: u32,
    pub present_family: u32,
}

impl VkQueues {
    #[instrument(level = "trace", skip_all)]
    pub fn find_queue_families(
        instance: &Instance,
        surface_loader: &ash::khr::surface::Instance,
        surface: SurfaceKHR,
        physical_device: PhysicalDevice,
    ) -> Result<Option<VkQueues>> {
        let queue_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let mut queue = None;
        let mut present = None;

        for (i, prop) in queue_family_properties.iter().enumerate() {
            let i = i as u32;
            if prop.queue_flags.contains(QueueFlags::GRAPHICS) {
                queue = Some(i);
            }

            let present_ok = unsafe {
                surface_loader.get_physical_device_surface_support(physical_device, i, surface)?
            };
            if present_ok {
                present = Some(i);
            }
            if queue.is_some() && present.is_some() {
                break;
            }
        }
        Ok(queue.and_then(|gg| {
            present.map(|pp| VkQueues {
                graphics_family: gg,
                present_family: pp,
            })
        }))
    }
}
