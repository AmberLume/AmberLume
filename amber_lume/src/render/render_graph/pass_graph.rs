use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::image_state_tracker::image_state_tracker::ImageStateTracker;
use crate::render::render_graph::pass_entry::concrete_pass_entry::ConcretePassEntry;
use crate::render::render_graph::pass_entry::pass_entry::PassEntry;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use anyhow::Result;
use ash::vk::{Extent2D, Image, ImageSubresourceRange, ImageView};
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::statistics::pass_profiler::PassProfiler;
use crate::resources::dynamic::image::image_backend::ImageBackend;
use crate::resources::dynamic::resource_provider::{ResourceId, ResourceProvider};

pub struct PassGraph {
    passes: Vec<Box<dyn PassEntry>>,
    declaration: PassResourceDeclaration,
    pub resource_registry: ResourceRegistry,
}

impl PassGraph {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            declaration: PassResourceDeclaration::new(),
            resource_registry: ResourceRegistry::new(),
        }
    }

    pub fn create_image(&mut self, label: &'static str, blueprint: ImageBlueprint) -> VirtualImage {
        self.resource_registry.create_image(label, blueprint)
    }

    pub fn import_image(
        &mut self,
        image: Image,
        image_view: ImageView,
        layers: Vec<ImageView>,
        extent: Extent2D,
        subresource_range: ImageSubresourceRange,
        descriptor_id: Option<ResourceId>,
    ) -> VirtualImage {
        self.resource_registry.import_image(image, image_view, layers, extent, subresource_range, descriptor_id)
    }

    pub fn import_image_placeholder(
        &mut self,
    ) -> VirtualImage {
        self.resource_registry.import_image_placeholder()
    }

    pub fn add_pass<P: Pass + 'static>(&mut self, pass: P) {
        self.passes.push(Box::new(ConcretePassEntry::new(pass)));
    }

    pub fn update_imported(
        &mut self,
        handle: VirtualImage,
        image: Image,
        image_view: ImageView,
        layers: Vec<ImageView>,
        extent: Extent2D,
        subresource_range: ImageSubresourceRange,
    ) {
        self.resource_registry.update_imported(handle, image, image_view, layers, extent, subresource_range)
    }

    pub fn build(
        &mut self,
        swapchain_extent: Extent2D,
        resource_factories: &ResourceFactories,
        image_provider: &ResourceProvider<ImageBackend>,
    ) -> Result<()> {
        self.resource_registry.build(swapchain_extent, &resource_factories.managed_image_factory, &image_provider)
    }

    pub fn run(
        &mut self,
        frame_data_context: &FrameDataContext,
        pass_context: &PassContext,
        image_state_tracker: &mut ImageStateTracker,
        pass_profiler: &mut PassProfiler,
    ) -> Result<()> {
        for entry in &mut self.passes {
            entry.run(
                frame_data_context,
                pass_context,
                &mut self.declaration,
                image_state_tracker,
                &mut self.resource_registry,
                pass_profiler,
            )?;
        }

        Ok(())
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        for entry in self.passes {
            entry.destroy(resource_factories)?;
        }

        self.resource_registry.destroy(&resource_factories.managed_image_factory)?;

        Ok(())
    }
}
