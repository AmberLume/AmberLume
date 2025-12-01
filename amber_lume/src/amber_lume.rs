use crate::providers::Providers;
use crate::render::context_profile::ContextProfile;
use crate::render::vulkan::buffer::resource_context::ResourceContext;
use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::render_context::RenderContext;
use crate::render::vulkan::vk_context::VkContext;
use crate::render::vulkan::vk_surface::VkSurface;
use crate::resources::resource_hub::ResourceHub;
use anyhow::Result;
use std::sync::Arc;

pub struct AmberLume {
    vk_context: Arc<VkContext>,
    device_context: Arc<DeviceContext>,
    resource_context: ResourceContext,
    render_context: RenderContext,

    providers: Providers,

    resource_hub: ResourceHub,
}

impl AmberLume {
    pub fn new(providers: Providers) -> Result<Self> {
        let context_profile = ContextProfile::from(providers.surface_provider.clone())?;

        let vk_context = Arc::new(VkContext::new(context_profile)?);

        let vk_surface = {
            let vk_surface =
                VkSurface::create(vk_context.clone(), providers.surface_provider.clone())?;

            Arc::new(vk_surface)
        };

        let device_context = {
            let device_context = DeviceContext::new(&vk_context, &vk_surface)?;

            Arc::new(device_context)
        };

        let render_context = RenderContext::create(
            vk_context.clone(),
            vk_surface.clone(),
            device_context.clone(),
            providers.surface_provider.clone(),
        )?;

        let resource_context = ResourceContext::create(device_context.clone())?;

        let resource_hub =
            ResourceHub::new(device_context.device.clone(), providers.io_provider.clone());

        Ok(Self {
            vk_context,
            device_context,
            render_context,
            resource_context,

            providers,

            resource_hub,
        })
    }

    pub fn render(&mut self) -> Result<()> {
        self.render_context.begin_frame()
    }

    pub fn resize(&mut self) {
        self.render_context.request_recreate_swapchain();
    }

    pub fn destroy(&mut self) -> Result<()> {
        self.render_context.destroy();
        self.resource_context.destroy();
        self.device_context.destroy();
        Ok(())
    }
}
