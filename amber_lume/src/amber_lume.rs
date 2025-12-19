use crate::data::providers::Providers;
use crate::render::context_profile::ContextProfile;
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::buffer::resource_context::ResourceContext;
use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::renderer::renderer::Renderer;
use crate::render::vulkan::surface::vulkan_surface::VulkanSurface;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use crate::render::vulkan::vulkan_context::VulkanContext;
use crate::resources::resource_hub::ResourceHub;
use anyhow::Result;
use std::sync::Arc;
use tracing::info;

pub struct AmberLume {
    vulkan_context: Arc<VulkanContext>,
    vulkan_surface: VulkanSurface,

    device_context: DeviceContext,
    swapchain_context: SwapchainContext,

    renderer: Renderer,

    resource_context: ResourceContext,

    providers: Providers,

    resource_hub: Arc<ResourceHub>,

    buffer_manager: BufferManager,
}

impl AmberLume {
    pub fn new(providers: Providers) -> Result<Self> {
        let context_profile = ContextProfile::from(providers.surface_provider.clone())?;

        let vulkan_context = Arc::new(VulkanContext::new(context_profile)?);

        let vulkan_surface =
            VulkanSurface::create(&vulkan_context, providers.surface_provider.clone())?;

        let mut device_context = DeviceContext::new(&vulkan_context, &vulkan_surface)?;

        let swapchain_context = SwapchainContext::create(
            &vulkan_context,
            &vulkan_surface,
            &device_context,
            providers.surface_provider.clone(),
        )?;

        let buffer_manager = BufferManager::create(&mut device_context);

        let mut resource_context = ResourceContext::create(&mut device_context)?;
        let resource_hub = {
            let resource_hub = ResourceHub::create(
                &mut device_context,
                &mut resource_context,
                &buffer_manager,
                providers.io_provider.clone(),
            )?;

            Arc::new(resource_hub)
        };

        let renderer = Renderer::create(
            &vulkan_context,
            &mut device_context,
            &swapchain_context,
            resource_hub.clone(),
            &buffer_manager,
        )?;

        info!("AmberLume created");

        Ok(Self {
            vulkan_context,
            vulkan_surface,

            device_context,
            swapchain_context,

            renderer,

            resource_context,

            providers,

            resource_hub,

            buffer_manager,
        })
    }

    pub fn render(&mut self) -> Result<()> {
        self.renderer
            .render_frame(&self.device_context, &self.swapchain_context)?;

        Ok(())
    }

    pub fn resize(&mut self) -> Result<()> {
        info!("Resize started");

        self.renderer.teardown(&mut self.device_context)?;

        self.swapchain_context.teardown_and_setup(
            &self.vulkan_context,
            &self.vulkan_surface,
            &mut self.device_context,
            self.providers.surface_provider.clone(),
        )?;

        self.renderer.setup(
            &self.vulkan_context,
            &mut self.device_context,
            &self.swapchain_context,
        )?;

        info!("Resized successfully");

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        info!("Stop running");

        self.destroy()?;

        info!("AmberLume stopped gracefully");

        Ok(())
    }

    pub fn destroy(&mut self) -> Result<()> {
        unsafe { self.device_context.device.device_wait_idle()? };

        self.renderer.destroy(&mut self.device_context)?;
        self.resource_hub.destroy()?;

        self.buffer_manager.destroy()?;

        self.swapchain_context.destroy(&mut self.device_context)?;
        self.resource_context.destroy(&self.device_context)?;
        self.device_context.destroy()?;

        self.vulkan_surface.destroy(&self.vulkan_context)?;
        self.vulkan_context.destroy()?;

        info!("AmberLume destroyed");

        Ok(())
    }
}
