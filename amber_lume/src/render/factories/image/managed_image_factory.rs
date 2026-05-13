use std::mem::ManuallyDrop;
use std::sync::Arc;
use ash::Device;
use gpu_allocator::vulkan::Allocator;
use crate::render::factories::image::managed_image::ManagedImage;
use anyhow::{bail, Result};
use parking_lot::Mutex;
use tracing::info;
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::utils::debug_utils::DebugUtils;
use crate::render::factories::image::image_utils::{create_allocation, create_image, create_image_subresource_range, create_image_view};
use crate::render::factories::image::image_view_description::ImageViewDescription;

pub struct ManagedImageFactory {
    device: Device,
    allocator: Arc<Mutex<ManuallyDrop<Allocator>>>,
    debug_utils: Arc<DebugUtils>,
}

impl ManagedImageFactory {
    pub fn new(
        device: Device,
        allocator: Arc<Mutex<ManuallyDrop<Allocator>>>,
        debug_utils: Arc<DebugUtils>,
    ) -> Self {
        Self {
            device,
            allocator,
            debug_utils,
        }
    }
    
    pub fn allocate(
        &self, 
        label: &str,
        image_description: ImageDescription,
        image_view_description: ImageViewDescription,
    ) -> Result<ManagedImage> {
        let image = create_image(&self.device, &image_description)?;

        let allocation_result = create_allocation(
            &self.device,
            self.allocator.clone(),
            &format!("image_allocation_{}", label),
            image,
        );

        let allocation = if let Ok(allocation) = allocation_result {
            allocation
        } else {
            unsafe { self.device.destroy_image(image, None) };

            bail!("Failed to allocate image memory")
        };

        let image_subresource_range = create_image_subresource_range(&image_view_description);

        let image_view_result = create_image_view(
            &self.device,
            image,
            image_description.format,
            &image_view_description,
            image_subresource_range,
        );

        let image_view = if let Ok(image_view) = image_view_result {
            image_view
        } else {
            unsafe { self.device.destroy_image(image, None) };

            self.allocator.lock().free(allocation)?;

            bail!("Failed to create image view")
        };

        self.debug_utils.label(image, &format!("managed_image_{}", label));
        self.debug_utils.label(image_view, &format!("managed_image_view_{}", label));

        info!("ManagedImage '{}' created", label);

        Ok(ManagedImage {
            label: label.to_string(),

            image_description,
            image_view_description,

            image,
            image_view,

            image_subresource_range,
            allocation,
        })
    }

    pub fn destroy_image(&self, managed_image: ManagedImage) -> Result<()> {
        unsafe { self.device.destroy_image_view(managed_image.image_view, None) };

        unsafe { self.device.destroy_image(managed_image.image, None) };

        self.allocator.lock().free(managed_image.allocation)?;

        info!("ManagedImage '{}' destroyed", managed_image.label);

        Ok(())
    }
}
