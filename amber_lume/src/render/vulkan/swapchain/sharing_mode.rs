use crate::render::vulkan::queue::queue_families::QueueFamilies;
use ash::vk::SharingMode;
use tracing::info;

pub fn get_sharing_mode(queue_families: &QueueFamilies) -> SharingMode {
    let sharing_mode = if queue_families.graphics == queue_families.present {
        SharingMode::EXCLUSIVE
    } else {
        SharingMode::CONCURRENT
    };

    info!("Selected SharingMode: {:?}", sharing_mode);

    sharing_mode
}
