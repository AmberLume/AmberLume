use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};
use ash::Device;
use gpu_allocator::vulkan::Allocator;
use crate::render::factories::image::managed_image::{ImageDescription, ImageViewDescription, ManagedImage};
use anyhow::{bail, Result};
use ash::vk::ImageViewType;
use tracing::info;
use crate::render::utils::debug_utils::DebugUtils;
use crate::render::factories::image::image_utils::{create_allocation, create_image, create_image_subresource_range, create_image_view};

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

            if let Ok(mut allocator) = self.allocator.lock() {
                allocator.free(allocation)?;
            }

            bail!("Failed to create image view")
        };

        let mut image_view_layers = Vec::new();
        if image_view_description.layered {
            for i in 0..image_view_description.layer_count {
                let layer_image_view_description = ImageViewDescription {
                    image_view_type: ImageViewType::TYPE_2D,

                    base_array_layer: i,
                    layer_count: 1,

                    ..image_view_description
                };

                let image_subresource_range = create_image_subresource_range(&layer_image_view_description);

                let layer_image_view = create_image_view(
                    &self.device,
                    image,
                    image_description.format,
                    &layer_image_view_description,
                    image_subresource_range,
                )?;

                self.debug_utils.label(layer_image_view, &format!("managed_image_view_{}_layer_{}", label, i));

                image_view_layers.push(layer_image_view);
            }
        }

        self.debug_utils.label(image, &format!("managed_image_{}", label));
        self.debug_utils.label(image_view, &format!("managed_image_view_{}", label));
        
        info!("ManagedImage '{}' created", label);
        
        Ok(ManagedImage {
            label: label.to_string(),

            image_description,
            image_view_description,

            image,
            image_view,
            image_view_layers,

            image_subresource_range,
            allocation,
        })
    }
    
    pub fn destroy_image(&self, managed_image: ManagedImage) -> Result<()> {
        for layer_image_view in managed_image.image_view_layers {
            unsafe { self.device.destroy_image_view(layer_image_view, None) };
        }
        unsafe { self.device.destroy_image_view(managed_image.image_view, None) };

        unsafe { self.device.destroy_image(managed_image.image, None) };

        if let Ok(mut allocator) = self.allocator.lock() {
            allocator.free(managed_image.allocation)?;
        }

        info!("ManagedImage '{}' destroyed", managed_image.label);

        Ok(())
    }
}
