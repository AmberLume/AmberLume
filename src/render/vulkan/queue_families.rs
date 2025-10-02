use super::instance_surface::InstanceSurface;
use anyhow::Result;
use ash::vk;
use tracing::{info, instrument};
use vk::{PhysicalDevice, QueueFlags};

#[derive(Clone, Copy, Debug)]
pub struct QueueFamilies {
    pub graphics: u32,
    pub present: u32,
}

impl QueueFamilies {
    #[instrument(level = "trace", skip(instance_surface))]
    pub fn find(
        instance_surface: &InstanceSurface,
        physical_device: PhysicalDevice,
    ) -> Result<Self> {
        let queue_family_properties = unsafe {
            instance_surface
                .instance
                .get_physical_device_queue_family_properties(physical_device)
        };
        let mut graphics = None;
        let mut present = None;

        info!("Searching for graphics queue families");
        for (i, properties) in queue_family_properties.iter().enumerate() {
            if properties.queue_flags.contains(QueueFlags::GRAPHICS) {
                graphics = Some(i);
            }
            let ok = unsafe {
                instance_surface
                    .surface_loader
                    .get_physical_device_surface_support(
                        physical_device,
                        i as u32,
                        instance_surface.surface,
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
