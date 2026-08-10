use crate::resource_state_tracker::resource_state_tracker::ResourceStateTracker;
use crate::pass_resource_declaration::image_transition_declaration::ImageTransitionDeclaration;
use crate::virtual_image::image_subresource::ImageSubresource;
use crate::virtual_image::physical_image::PhysicalImage;
use crate::virtual_image::virtual_image::VirtualImage;
use ahash::HashSet;
use ash::vk::{AccessFlags, ImageLayout, ImageSubresourceRange, PipelineStageFlags};
use crate::pass_resource_declaration::buffer_transition_declaration::BufferTransitionDeclaration;
use crate::pass_resource_declaration::acceleration_structure_transition_declaration::AccelerationStructureTransitionDeclaration;
use crate::virtual_acceleration_structure::virtual_acceleration_structure::VirtualAccelerationStructure;
use crate::virtual_buffer::physical_buffer::PhysicalBuffer;
use crate::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::virtual_data::data_key::DataKey;
use crate::virtual_data::virtual_data::VirtualData;

pub struct PassResourceDeclaration {
    image_reads: Vec<ImageTransitionDeclaration>,
    image_writes: Vec<ImageTransitionDeclaration>,

    buffer_reads: Vec<BufferTransitionDeclaration>,
    buffer_writes: Vec<BufferTransitionDeclaration>,

    acceleration_structure_reads: Vec<AccelerationStructureTransitionDeclaration>,
    acceleration_structure_writes: Vec<AccelerationStructureTransitionDeclaration>,

    data_reads: Vec<DataKey>,
    data_writes: Vec<DataKey>,
}

impl PassResourceDeclaration {
    pub fn new() -> Self {
        Self {
            image_reads: Vec::new(),
            image_writes: Vec::new(),

            buffer_reads: Vec::new(),
            buffer_writes: Vec::new(),

            acceleration_structure_reads: Vec::new(),
            acceleration_structure_writes: Vec::new(),

            data_reads: Vec::new(),
            data_writes: Vec::new(),
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
            None,
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
            None,
            layout,
            access,
            stage,
        ));

        self
    }

    pub fn read_image_mip(
        &mut self,
        image: VirtualImage,
        mip: u32,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> &mut Self {
        self.image_reads.push(ImageTransitionDeclaration::new(
            image,
            Some(mip),
            layout,
            access,
            stage,
        ));

        self
    }

    pub fn write_image_mip(
        &mut self,
        image: VirtualImage,
        mip: u32,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> &mut Self {
        self.image_writes.push(ImageTransitionDeclaration::new(
            image,
            Some(mip),
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

    pub fn read_acceleration_structure(
        &mut self,
        acceleration_structure: VirtualAccelerationStructure,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> &mut Self {
        self.acceleration_structure_reads.push(AccelerationStructureTransitionDeclaration::new(
            acceleration_structure,
            access,
            stage,
        ));

        self
    }

    pub fn write_acceleration_structure(
        &mut self,
        acceleration_structure: VirtualAccelerationStructure,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> &mut Self {
        self.acceleration_structure_writes.push(AccelerationStructureTransitionDeclaration::new(
            acceleration_structure,
            access,
            stage,
        ));

        self
    }

    pub fn consume<T>(&mut self, data: VirtualData<T>) -> &mut Self {
        self.data_reads.push(data.key);

        self
    }

    pub fn produce<T>(&mut self, data: VirtualData<T>) -> &mut Self {
        self.data_writes.push(data.key);

        self
    }

    pub fn apply(
        &self,
        tracker: &mut ResourceStateTracker,
        image_resolver: &impl Fn(VirtualImage) -> PhysicalImage,
        buffer_resolver: &impl Fn(VirtualBuffer) -> PhysicalBuffer,
    ) {
        let written_images: HashSet<(VirtualImage, Option<u32>)> = self.image_writes.iter()
            .map(|declaration| (declaration.image, declaration.mip))
            .collect();
        let written_buffers: HashSet<VirtualBuffer> = self.buffer_writes.iter()
            .map(|declaration| declaration.buffer)
            .collect();

        let image_reads = self.image_reads.iter()
            .filter(|declaration| !written_images.contains(&(declaration.image, declaration.mip)));
        for image_declaration in image_reads.chain(self.image_writes.iter()) {
            let physical_image = image_resolver(image_declaration.image);

            let subresource_range = match image_declaration.mip {
                Some(base_mip_level) => ImageSubresourceRange::default()
                    .aspect_mask(physical_image.subresource_range.aspect_mask)
                    .base_mip_level(base_mip_level)
                    .level_count(1)
                    .base_array_layer(physical_image.subresource_range.base_array_layer)
                    .layer_count(physical_image.subresource_range.layer_count),
                None => physical_image.subresource_range,
            };

            tracker.image_transition(
                physical_image.image,
                subresource_range,
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

        let written_acceleration_structures: HashSet<VirtualAccelerationStructure> = self.acceleration_structure_writes.iter()
            .map(|declaration| declaration.acceleration_structure)
            .collect();

        let acceleration_structure_reads = self.acceleration_structure_reads.iter()
            .filter(|declaration| !written_acceleration_structures.contains(&declaration.acceleration_structure));
        for acceleration_structure_declaration in acceleration_structure_reads.chain(self.acceleration_structure_writes.iter()) {
            tracker.acceleration_structure_transition(
                acceleration_structure_declaration.acceleration_structure,
                acceleration_structure_declaration.access,
                acceleration_structure_declaration.stage,
            );
        }
    }

    pub fn read_images(&self) -> impl Iterator<Item = ImageSubresource> + '_ {
        self.image_reads.iter().map(|declaration| ImageSubresource {
            image: declaration.image,
            mip: declaration.mip,
        })
    }

    pub fn write_images(&self) -> impl Iterator<Item = ImageSubresource> + '_ {
        self.image_writes.iter().map(|declaration| ImageSubresource {
            image: declaration.image,
            mip: declaration.mip,
        })
    }

    pub fn read_buffers(&self) -> impl Iterator<Item = VirtualBuffer> + '_ {
        self.buffer_reads.iter().map(|buffer| buffer.buffer)
    }

    pub fn write_buffers(&self) -> impl Iterator<Item = VirtualBuffer> + '_ {
        self.buffer_writes.iter().map(|buffer| buffer.buffer)
    }

    pub fn read_acceleration_structures(&self) -> impl Iterator<Item = VirtualAccelerationStructure> + '_ {
        self.acceleration_structure_reads.iter().map(|declaration| declaration.acceleration_structure)
    }

    pub fn write_acceleration_structures(&self) -> impl Iterator<Item = VirtualAccelerationStructure> + '_ {
        self.acceleration_structure_writes.iter().map(|declaration| declaration.acceleration_structure)
    }

    pub fn read_data(&self) -> impl Iterator<Item = DataKey> + '_ {
        self.data_reads.iter().copied()
    }

    pub fn write_data(&self) -> impl Iterator<Item = DataKey> + '_ {
        self.data_writes.iter().copied()
    }

    pub fn clear(&mut self) {
        self.image_reads.clear();
        self.image_writes.clear();

        self.buffer_reads.clear();
        self.buffer_writes.clear();

        self.acceleration_structure_reads.clear();
        self.acceleration_structure_writes.clear();

        self.data_reads.clear();
        self.data_writes.clear();
    }
}
