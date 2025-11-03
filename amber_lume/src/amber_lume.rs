use crate::providers::Providers;
use crate::render::context_profile::ContextProfile;
use crate::render::vulkan::render_context::RenderContext;
use crate::render::vulkan::vk_context::VkContext;
use crate::resources::resource_hub::ResourceHub;
use anyhow::Result;
use std::sync::Arc;

pub struct AmberLume {
    vk_context: Arc<VkContext>,
    render_context: RenderContext,

    providers: Providers,

    resource_hub: ResourceHub,
}

impl AmberLume {
    pub fn new(providers: Providers) -> Result<Self> {
        let context_profile = ContextProfile::from(providers.surface_provider.clone())?;

        let vk_context = Arc::new(VkContext::new(context_profile)?);

        let render_context =
            RenderContext::create_from(vk_context.clone(), providers.surface_provider.clone())?;

        let arc_device = Arc::new(render_context.device.clone());

        let resource_hub = ResourceHub::new(arc_device.clone(), providers.io_provider.clone());

        Ok(Self {
            vk_context,
            render_context,

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
        Ok(())
    }
}
