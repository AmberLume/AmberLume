use crate::render::vulkan::queue::queue_families::QueueFamilies;
use ash::vk::SharingMode;
use tracing::{info, warn};

pub fn get_queue_family_indices(
    sharing_mode: SharingMode,
    queue_families: &QueueFamilies,
) -> Vec<u32> {
    let queue_family_indices = match sharing_mode {
        SharingMode::EXCLUSIVE => vec![queue_families.graphics],
        SharingMode::CONCURRENT => vec![queue_families.graphics, queue_families.present],
        _ => {
            warn!("Unexpected sharing mode {:?}", sharing_mode);

            vec![]
        }
    };

    info!("[QueueFamily] indices: {:?}", queue_family_indices);

    queue_family_indices
}
