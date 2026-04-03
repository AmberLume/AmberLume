use crate::render::render_graph::image_state_tracker::image_state_tracker::ImageStateTracker;
use crate::render::render_graph::pass_resource_declaration::image_transition_declaration::ImageTransitionDeclaration;
use ash::vk::{AccessFlags, Image, ImageLayout, ImageSubresourceRange, PipelineStageFlags};

pub struct PassResourceDeclaration {
    images: Vec<ImageTransitionDeclaration>,
}

impl PassResourceDeclaration {
    pub fn new() -> Self {
        Self { images: Vec::new() }
    }

    pub fn image(
        &mut self,
        image: Image,
        subresource_range: ImageSubresourceRange,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> &mut Self {
        self.images.push(ImageTransitionDeclaration {
            image,
            subresource_range,
            layout,
            access,
            stage,
        });

        self
    }

    pub fn apply(&self, tracker: &mut ImageStateTracker) {
        for declaration in &self.images {
            tracker.transition(
                declaration.image,
                declaration.subresource_range,
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
