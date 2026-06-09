use anyhow::{Result, anyhow, bail};
use ash::vk::{self, Extent2D, Fence, Format, PresentInfoKHR, Semaphore, SemaphoreCreateInfo};
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use ash::Device;
use tracing::info;

use crate::platform_providers::surface_provider::SurfaceProvider;
use crate::render::device::device_context::DeviceContext;
use crate::render::device::vulkan_context::VulkanContext;
use crate::render::queue::queues::Queues;
use crate::render::surface::render_surface::RenderSurface;
use crate::render::swapchain::surface_format::{log_hdr_support, query_surface_formats, surface_supports_hdr, HDR_FORMAT};
use crate::render::swapchain::swapchain_context::SwapchainContext;
use crate::render::target::render_target::{RenderTarget, RenderTargetImage};

struct Inner {
    swapchain_context: SwapchainContext,
    present_semaphores: Vec<Semaphore>,
}

pub struct SurfaceRenderTarget {
    surface_provider: Arc<dyn SurfaceProvider>,
    render_surface: RenderSurface,

    inner: Mutex<Option<Inner>>,

    is_out_of_date: AtomicBool,

    hdr_supported: bool,
}

impl SurfaceRenderTarget {
    pub fn create(
        vulkan_context: &VulkanContext,
        device_context: &DeviceContext,
        surface_provider: Arc<dyn SurfaceProvider>,
        hdr: bool,
    ) -> Result<Self> {
        let render_surface = RenderSurface::create(vulkan_context, surface_provider.clone())?;
        device_context
            .physical_device_info
            .validate_surface(vulkan_context, &render_surface)?;
        device_context.attach_present(vulkan_context, render_surface.surface)?;

        let surface_formats = query_surface_formats(
            vulkan_context,
            &render_surface,
            &device_context.physical_device_info,
        )?;
        let hdr_supported = surface_supports_hdr(&surface_formats);
        log_hdr_support(&surface_formats);

        let swapchain_context = SwapchainContext::create(
            None,
            vulkan_context,
            &render_surface,
            device_context,
            surface_provider.clone(),
            hdr && hdr_supported,
        )?;
        let present_semaphores = Self::create_semaphores(
            &device_context.device,
            swapchain_context.swapchain_images.len(),
        )?;

        info!("SurfaceRenderTarget created");

        Ok(Self {
            surface_provider,
            render_surface,
            inner: Mutex::new(Some(Inner {
                swapchain_context,
                present_semaphores,
            })),
            is_out_of_date: AtomicBool::new(false),
            hdr_supported,
        })
    }

    fn create_semaphores(device: &Device, count: usize) -> Result<Vec<Semaphore>> {
        let info = SemaphoreCreateInfo::default();

        (0..count)
            .map(|_| Ok(unsafe { device.create_semaphore(&info, None)? }))
            .collect()
    }

    fn destroy_semaphores(device: &Device, semaphores: &[Semaphore]) {
        for &semaphore in semaphores {
            unsafe { device.destroy_semaphore(semaphore, None) };
        }
    }

    fn with_inner<R>(&self, f: impl FnOnce(&Inner) -> Result<R>) -> Result<R> {
        let guard = self.inner.lock();
        let inner = guard.as_ref().ok_or_else(|| anyhow!("RenderTarget already destroyed"))?;

        f(inner)
    }
}

impl RenderTarget for SurfaceRenderTarget {
    fn format(&self) -> Format {
        self.with_inner(|inner| Ok(inner.swapchain_context.format))
            .expect("RenderTarget destroyed")
    }

    fn extent(&self) -> Extent2D {
        self.with_inner(|inner| Ok(inner.swapchain_context.extent))
            .expect("RenderTarget destroyed")
    }

    fn image_count(&self) -> u32 {
        self.with_inner(|inner| Ok(inner.swapchain_context.swapchain_images.len() as u32))
            .expect("RenderTarget destroyed")
    }

    fn acquire_next_image(&self, signal_semaphore: Semaphore) -> Result<Option<u32>> {
        let acquire_result = self.with_inner(|inner| {
            let swapchain = &inner.swapchain_context;

            Ok(unsafe {
                swapchain.loader.acquire_next_image(
                    swapchain.handle,
                    u64::MAX,
                    signal_semaphore,
                    Fence::null(),
                )
            })
        })?;

        match acquire_result {
            Ok((image_index, suboptimal)) => {
                if suboptimal {
                    self.is_out_of_date.store(true, Ordering::Relaxed);
                }
                Ok(Some(image_index))
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.is_out_of_date.store(true, Ordering::Relaxed);
                info!("Swapchain acquire out of date");
                Ok(None)
            }
            Err(error) => bail!(error),
        }
    }

    fn get_image(&self, index: u32) -> Result<RenderTargetImage> {
        self.with_inner(|inner| {
            let image = inner.swapchain_context.get_image(index)?;

            Ok(RenderTargetImage {
                image: image.image,
                image_view: image.image_view,
                format: image.format,
                extent: image.extent,
                image_subresource_range: image.image_subresource_range,
            })
        })
    }

    fn get_present_semaphore(&self, index: u32) -> Result<Semaphore> {
        self.with_inner(|inner| {
            inner
                .present_semaphores
                .get(index as usize)
                .copied()
                .ok_or_else(|| anyhow!("Present semaphore index out of bounds"))
        })
    }

    fn present(&self, queues: &Queues, image_index: u32, wait_semaphore: Semaphore) -> Result<()> {
        let queue = queues.present_queue();

        let result = self.with_inner(|inner| {
            let swapchain = &inner.swapchain_context;

            let wait_semaphores = [wait_semaphore];
            let swapchains = [swapchain.handle];
            let image_indices = [image_index];
            let present_info = PresentInfoKHR::default()
                .wait_semaphores(&wait_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            let queue_guard = queue.queue.lock();

            Ok(unsafe { swapchain.loader.queue_present(*queue_guard, &present_info) })
        })?;

        match result {
            Ok(true)
            | Err(vk::Result::ERROR_OUT_OF_DATE_KHR)
            | Err(vk::Result::ERROR_SURFACE_LOST_KHR) => {
                self.is_out_of_date.store(true, Ordering::Relaxed);
                
                info!("Swapchain present out of date");
            }
            Ok(false) => {}
            Err(error) => bail!(error),
        }

        Ok(())
    }

    fn is_out_of_date(&self) -> bool {
        self.is_out_of_date.load(Ordering::Relaxed)
    }

    fn set_out_of_date(&self, value: bool) {
        self.is_out_of_date.store(value, Ordering::Relaxed);
    }

    fn hdr_supported(&self) -> bool {
        self.hdr_supported
    }

    fn is_hdr(&self) -> bool {
        self.format() == HDR_FORMAT
    }

    fn invalidate(
        &self,
        vulkan_context: &VulkanContext,
        device_context: &DeviceContext,
        hdr: bool,
    ) -> Result<()> {
        info!("Invalidating SurfaceRenderTarget");

        device_context.queues.present_wait_idle()?;

        let mut guard = self.inner.lock();
        let old = guard.take().ok_or_else(|| anyhow!("RenderTarget already destroyed"))?;

        let new_swapchain = SwapchainContext::create(
            Some(&old.swapchain_context),
            vulkan_context,
            &self.render_surface,
            device_context,
            self.surface_provider.clone(),
            hdr && self.hdr_supported,
        )?;
        let new_semaphores = Self::create_semaphores(
            &device_context.device,
            new_swapchain.swapchain_images.len(),
        )?;

        *guard = Some(Inner {
            swapchain_context: new_swapchain,
            present_semaphores: new_semaphores,
        });

        Self::destroy_semaphores(&device_context.device, &old.present_semaphores);
        old.swapchain_context.destroy(&device_context.device)?;

        self.is_out_of_date.store(false, Ordering::Relaxed);

        info!("SurfaceRenderTarget invalidated");

        Ok(())
    }

    fn destroy_resources(
        &self, 
        vulkan_context: &VulkanContext, 
        device_context: &DeviceContext,
    ) -> Result<()> {
        device_context.queues.all_wait_idle()?;

        let inner = self.inner.lock().take();

        if let Some(inner) = inner {
            Self::destroy_semaphores(&device_context.device, &inner.present_semaphores);
            inner.swapchain_context.destroy(&device_context.device)?;
        }

        self.render_surface.destroy(vulkan_context)?;
        device_context.detach_present();

        info!("SurfaceRenderTarget destroyed");

        Ok(())
    }
}
