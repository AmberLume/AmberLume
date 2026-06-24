use std::mem::ManuallyDrop;
use std::sync::Arc;
use ash::vk::{Format, Image, ImageCreateInfo, ImageLayout, ImageSubresourceRange, ImageView, ImageViewCreateInfo, SampleCountFlags};
use anyhow::Result;
use ash::Device;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use parking_lot::Mutex;
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::factories::image::image_view_description::ImageViewDescription;

pub fn create_image(
    device: &Device,
    description: &ImageDescription,
) -> Result<Image> {
    let image_info = ImageCreateInfo::default()
        .flags(description.flags)
        .image_type(description.image_type)
        .format(description.format)
        .extent(description.extent)
        .mip_levels(description.mip_levels)
        .array_layers(description.array_layers)
        .samples(SampleCountFlags::TYPE_1)
        .tiling(description.tiling)
        .usage(description.usage)
        .sharing_mode(description.sharing_mode)
        .initial_layout(ImageLayout::UNDEFINED);

    Ok(unsafe { device.create_image(&image_info, None)? })
}

pub fn create_image_view(
    device: &Device,
    image: Image,
    format: Format,
    image_view_description: &ImageViewDescription,
    image_subresource_range: ImageSubresourceRange,
) -> Result<ImageView> {
    let image_create_info = ImageViewCreateInfo::default()
        .image(image)
        .view_type(image_view_description.image_view_type)
        .format(format)
        .subresource_range(image_subresource_range);

    Ok(unsafe { device.create_image_view(&image_create_info, None)? })
}

pub fn create_image_subresource_range(image_view_description: &ImageViewDescription) -> ImageSubresourceRange {
    ImageSubresourceRange::default()
        .aspect_mask(image_view_description.image_aspect_flags)
        .base_mip_level(image_view_description.base_mip_level)
        .level_count(image_view_description.level_count)
        .base_array_layer(image_view_description.base_array_layer)
        .layer_count(image_view_description.layer_count)
}

pub fn create_allocation(
    device: &Device, 
    allocator: Arc<Mutex<ManuallyDrop<Allocator>>>,
    name: &str, 
    image: Image,
) -> Result<Allocation> {
    let requirements = unsafe { device.get_image_memory_requirements(image) };

    let allocation = allocator.lock().allocate(&AllocationCreateDesc {
        name: &format!("allocation_image_{}", name),
        requirements,
        location: MemoryLocation::GpuOnly,
        linear: false,
        allocation_scheme: AllocationScheme::GpuAllocatorManaged,
    })?;

    unsafe { device.bind_image_memory(image, allocation.memory(), allocation.offset())? };

    Ok(allocation)
}
