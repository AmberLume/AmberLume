use anyhow::Result;
use ash::vk::{Extent2D, Format, Image, ImageSubresourceRange, ImageView, PresentModeKHR, Semaphore};

use crate::device::device_context::DeviceContext;
use crate::device::vulkan_context::VulkanContext;
use crate::queue::queues::Queues;

#[derive(Copy, Clone)]
pub struct RenderTargetImage {
    pub image: Image,
    pub image_view: ImageView,
    pub format: Format,
    pub extent: Extent2D,
    pub image_subresource_range: ImageSubresourceRange,
}

pub trait RenderTarget: Send + Sync {
    fn format(&self) -> Format;

    fn extent(&self) -> Extent2D;

    fn image_count(&self) -> u32;

    fn acquire_next_image(&self, signal_semaphore: Semaphore) -> Result<Option<u32>>;

    fn get_image(&self, index: u32) -> Result<RenderTargetImage>;

    fn get_present_semaphore(&self, index: u32) -> Result<Semaphore>;

    fn present(&self, queues: &Queues, image_index: u32, wait_semaphore: Semaphore) -> Result<()>;

    fn is_out_of_date(&self) -> bool;

    fn set_out_of_date(&self, value: bool);

    fn hdr_supported(&self) -> bool;

    fn is_hdr(&self) -> bool;

    fn invalidate(&self, vulkan_context: &VulkanContext, device_context: &DeviceContext, hdr: bool, present_mode: PresentModeKHR) -> Result<()>;

    fn destroy_resources(&self, vulkan_context: &VulkanContext, device_context: &DeviceContext) -> Result<()>;
}
