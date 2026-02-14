use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};
use ash::Device;
use gpu_allocator::vulkan::Allocator;
use crate::render::vulkan::factories::image::managed_image::{ImageDescription, ImageViewDescription, ManagedImage};
use anyhow::Result;
use tracing::info;
use crate::render::vulkan::debug_utils::DebugUtils;
use crate::render::vulkan::factories::image::image_utils::{create_allocation, create_image, create_image_subresource_range, create_image_view};

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

        let allocation = create_allocation(
            &self.device,
            self.allocator.clone(),
            &format!("image_allocation_{}", label),
            image,
        )?;

        let image_subresource_range = create_image_subresource_range(&image_view_description);

        let image_view = create_image_view(
            &self.device,
            image,
            image_description.format,
            &image_view_description,
            image_subresource_range,
        )?;

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
    
    pub fn destroy(&self, managed_image: ManagedImage) -> Result<()> {
        unsafe { self.device.destroy_image_view(managed_image.image_view, None) };

        unsafe { self.device.destroy_image(managed_image.image, None) };

        if let Ok(mut allocator) = self.allocator.lock() {
            allocator.free(managed_image.allocation)?;
        }

        info!("ManagedImage '{}' destroyed", managed_image.label);

        Ok(())
    }
}
