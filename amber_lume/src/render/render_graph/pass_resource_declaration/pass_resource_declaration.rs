use crate::render::render_graph::resource_state_tracker::resource_state_tracker::ResourceStateTracker;
use crate::render::render_graph::pass_resource_declaration::image_transition_declaration::ImageTransitionDeclaration;
use crate::render::render_graph::virtual_image::physical_image::PhysicalImage;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use ahash::HashSet;
use ash::vk::{AccessFlags, ImageLayout, PipelineStageFlags};
use crate::render::render_graph::pass_resource_declaration::buffer_transition_declaration::BufferTransitionDeclaration;
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;

pub struct PassResourceDeclaration {
    image_reads: Vec<ImageTransitionDeclaration>,
    image_writes: Vec<ImageTransitionDeclaration>,

    buffer_reads: Vec<BufferTransitionDeclaration>,
    buffer_writes: Vec<BufferTransitionDeclaration>,
}

impl PassResourceDeclaration {
    pub fn new() -> Self {
        Self {
            image_reads: Vec::new(),
            image_writes: Vec::new(),

            buffer_reads: Vec::new(),
            buffer_writes: Vec::new(),
        }
    }

    pub fn read_image(
        &mut self,
        image: VirtualImage,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> &mut Self {
        self.image_reads.push(ImageTransitionDeclaration::new(
            image,
            layout,
            access,
            stage,
        ));

        self
    }

    pub fn write_image(
        &mut self,
        image: VirtualImage,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> &mut Self {
        self.image_writes.push(ImageTransitionDeclaration::new(
            image,
            layout,
            access,
            stage,
        ));

        self
    }

    pub fn read_buffer(
        &mut self,
        buffer: VirtualBuffer,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> &mut Self {
        self.buffer_reads.push(BufferTransitionDeclaration::new(
            buffer,
            access,
            stage,
        ));

        self
    }

    pub fn write_buffer(
        &mut self,
        buffer: VirtualBuffer,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> &mut Self {
        self.buffer_writes.push(BufferTransitionDeclaration::new(
            buffer,
            access,
            stage,
        ));

        self
    }

    pub fn apply(
        &self,
        tracker: &mut ResourceStateTracker,
        image_resolver: &impl Fn(VirtualImage) -> PhysicalImage,
        buffer_resolver: &impl Fn(VirtualBuffer) -> PhysicalBuffer,
    ) {
        let written_images: HashSet<VirtualImage> = self.image_writes.iter()
            .map(|declaration| declaration.image)
            .collect();
        let written_buffers: HashSet<VirtualBuffer> = self.buffer_writes.iter()
            .map(|declaration| declaration.buffer)
            .collect();

        let image_reads = self.image_reads.iter()
            .filter(|declaration| !written_images.contains(&declaration.image));
        for image_declaration in image_reads.chain(self.image_writes.iter()) {
            let physical_image = image_resolver(image_declaration.image);

            tracker.image_transition(
                physical_image.image,
                physical_image.subresource_range,
                image_declaration.layout,
                image_declaration.access,
                image_declaration.stage,
            );
        }

        let buffer_reads = self.buffer_reads.iter()
            .filter(|declaration| !written_buffers.contains(&declaration.buffer));
        for buffer_declaration in buffer_reads.chain(self.buffer_writes.iter()) {
            let physical_buffer = buffer_resolver(buffer_declaration.buffer);

            tracker.buffer_transition(
                physical_buffer.buffer,
                physical_buffer.offset,
                physical_buffer.size,
                buffer_declaration.access,
                buffer_declaration.stage,
            );
        }
    }

    pub fn read_images(&self) -> impl Iterator<Item = VirtualImage> + '_ {
        self.image_reads.iter().map(|image| image.image)
    }

    pub fn write_images(&self) -> impl Iterator<Item = VirtualImage> + '_ {
        self.image_writes.iter().map(|image| image.image)
    }

    pub fn read_buffers(&self) -> impl Iterator<Item = VirtualBuffer> + '_ {
        self.buffer_reads.iter().map(|buffer| buffer.buffer)
    }

    pub fn write_buffers(&self) -> impl Iterator<Item = VirtualBuffer> + '_ {
        self.buffer_writes.iter().map(|buffer| buffer.buffer)
    }

    pub fn clear(&mut self) {
        self.image_reads.clear();
        self.image_writes.clear();

        self.buffer_reads.clear();
        self.buffer_writes.clear();
    }
}
