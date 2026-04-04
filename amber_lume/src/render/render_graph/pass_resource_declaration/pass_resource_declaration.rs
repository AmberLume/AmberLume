use crate::render::render_graph::image_state_tracker::image_state_tracker::ImageStateTracker;
use crate::render::render_graph::pass_resource_declaration::image_transition_declaration::ImageTransitionDeclaration;
use crate::render::render_graph::virtual_image::physical_image::PhysicalImage;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use ash::vk::{AccessFlags, ImageLayout, PipelineStageFlags};

pub struct PassResourceDeclaration {
    reads: Vec<ImageTransitionDeclaration>,
    writes: Vec<ImageTransitionDeclaration>,
}

impl PassResourceDeclaration {
    pub fn new() -> Self {
        Self {
            reads: Vec::new(),
            writes: Vec::new(),
        }
    }

    pub fn read(
        &mut self,
        image: VirtualImage,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> &mut Self {
        self.reads.push(ImageTransitionDeclaration::new(
            image,
            layout,
            access,
            stage,
        ));

        self
    }

    pub fn write(
        &mut self,
        image: VirtualImage,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> &mut Self {
        self.writes.push(ImageTransitionDeclaration::new(
            image,
            layout,
            access,
            stage,
        ));

        self
    }

    pub fn apply(
        &self,
        tracker: &mut ImageStateTracker,
        resolver: &impl Fn(VirtualImage) -> PhysicalImage,
    ) {
        for declaration in self.reads.iter().chain(self.writes.iter()) {
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

    pub fn read_images(&self) -> impl Iterator<Item = VirtualImage> + '_ {
        self.reads.iter().map(|image| image.image)
    }

    pub fn write_images(&self) -> impl Iterator<Item = VirtualImage> + '_ {
        self.writes.iter().map(|image| image.image)
    }

    pub fn clear(&mut self) {
        self.reads.clear();
        self.writes.clear();
    }
}
