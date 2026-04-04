use crate::render::render_graph::image_state_tracker::image_state_tracker::ImageStateTracker;
use crate::render::render_graph::pass_resource_declaration::image_transition_declaration::ImageTransitionDeclaration;
use ash::vk::{AccessFlags, ImageLayout, PipelineStageFlags};
use crate::render::render_graph::virtual_image::physical_image::PhysicalImage;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;

pub struct PassResourceDeclaration {
    images: Vec<ImageTransitionDeclaration>,
}

impl PassResourceDeclaration {
    pub fn new() -> Self {
        Self { images: Vec::new() }
    }

    pub fn image(
        &mut self,
        image: VirtualImage,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> &mut Self {
        self.images.push(ImageTransitionDeclaration::new(image, layout, access, stage));

        self
    }

    pub fn apply(
        &self,
        tracker: &mut ImageStateTracker,
        resolver: &impl Fn(VirtualImage) -> PhysicalImage,
    ) {
        for declaration in &self.images {
            let physical_image = resolver(declaration.image);

            tracker.transition(
                physical_image.image,
                physical_image.subresource_range,
                declaration.layout,
                declaration.access,
                declaration.stage,
            );
        }
    }
    
    pub fn clear(&mut self) {
        self.images.clear();
    }
}
