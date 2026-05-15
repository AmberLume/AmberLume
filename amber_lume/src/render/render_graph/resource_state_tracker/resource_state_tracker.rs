use ash::vk::{AccessFlags, Buffer, BufferMemoryBarrier, DependencyFlags, DeviceSize, Image, ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, PipelineStageFlags, QUEUE_FAMILY_IGNORED};
use std::collections::HashMap;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::resource_state_tracker::buffer_region_key::BufferRegionKey;
use crate::render::render_graph::resource_state_tracker::buffer_region_state::BufferRegionState;
use crate::render::render_graph::resource_state_tracker::buffer_state::BufferState;
use crate::render::render_graph::resource_state_tracker::image_state::ImageState;
use crate::render::render_graph::resource_state_tracker::image_pending_barrier::PendingImageBarrier;
use crate::render::render_graph::resource_state_tracker::pending_buffer_barrier::PendingBufferBarrier;

pub struct ResourceStateTracker {
    image_transient_states: HashMap<Image, ImageState>,
    image_persistent_states: HashMap<Image, ImageState>,
    image_pending_barriers: Vec<PendingImageBarrier>,

    buffer_region_states: Vec<BufferRegionState>,
    buffer_pending_barriers: Vec<PendingBufferBarrier>,
}

impl ResourceStateTracker {
    pub fn new() -> Self {
        Self {
            image_transient_states: HashMap::new(),
            image_persistent_states: HashMap::new(),
            image_pending_barriers: Vec::new(),

            buffer_region_states: Vec::new(),
            buffer_pending_barriers: Vec::new(),
        }
    }

    pub fn register_persistent_image(
        &mut self,
        image: Image,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) {
        self.image_persistent_states.insert(image, ImageState { layout, access, stage });
    }

    pub fn begin_frame(&mut self) {
        debug_assert!(self.image_pending_barriers.is_empty(), "Unflushed image barriers at begin_frame");
        debug_assert!(self.buffer_pending_barriers.is_empty(), "Unflushed buffer barriers at begin_frame");
    }

    pub fn image_transition(
        &mut self,
        image: Image,
        subresource_range: ImageSubresourceRange,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) {
        let current = self.image_transient_states.get(&image)
            .or_else(|| self.image_persistent_states.get(&image))
            .copied()
            .unwrap_or_else(ImageState::undefined);

        let write_bits = AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
            | AccessFlags::COLOR_ATTACHMENT_WRITE
            | AccessFlags::SHADER_WRITE
            | AccessFlags::TRANSFER_WRITE
            | AccessFlags::HOST_WRITE
            | AccessFlags::MEMORY_WRITE;

        let both_read_only = !current.access.intersects(write_bits) && !access.intersects(write_bits);

        if current.layout == layout && both_read_only && current.access.contains(access) {
            return;
        }

        let state = ImageState { layout, access, stage };

        if self.image_persistent_states.contains_key(&image) {
            self.image_persistent_states.insert(image, state);
        } else {
            self.image_transient_states.insert(image, state);
        }

        self.image_pending_barriers.push(PendingImageBarrier {
            image,
            subresource_range,
            old_layout: current.layout,
            new_layout: layout,
            src_access: current.access,
            dst_access: access,
            src_stage: current.stage,
            dst_stage: stage,
        });
    }

    pub fn buffer_transition(
        &mut self,
        buffer: Buffer,
        offset: DeviceSize,
        size: DeviceSize,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) {
        let end = offset + size;

        let overlapping: Vec<usize> = self.buffer_region_states.iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.region.buffer == buffer
                    && entry.region.offset < end
                    && offset < entry.region.offset + entry.region.size
            })
            .map(|(index, _)| index)
            .collect();

        let current = if overlapping.is_empty() {
            BufferState::initial()
        } else {
            overlapping.iter().fold(
                BufferState {
                    access: AccessFlags::empty(),
                    stage: PipelineStageFlags::empty(),
                },
                |acc, &index| {
                    let entry = self.buffer_region_states[index].state;

                    BufferState {
                        access: acc.access | entry.access,
                        stage: acc.stage | entry.stage,
                    }
                },
            )
        };

        let write_bits = AccessFlags::SHADER_WRITE
            | AccessFlags::TRANSFER_WRITE
            | AccessFlags::HOST_WRITE
            | AccessFlags::MEMORY_WRITE;

        let both_read_only = !current.access.intersects(write_bits) && !access.intersects(write_bits);

        if !overlapping.is_empty() && both_read_only && current.access.contains(access) {
            return;
        }

        for &index in overlapping.iter().rev() {
            self.buffer_region_states.swap_remove(index);
        }

        self.buffer_region_states.push(BufferRegionState {
            region: BufferRegionKey { buffer, offset, size },
            state: BufferState { access, stage },
        });

        self.buffer_pending_barriers.push(PendingBufferBarrier {
            buffer,
            offset,
            size,
            src_access: current.access,
            dst_access: access,
            src_stage: current.stage,
            dst_stage: stage,
        });
    }

    pub fn flush(&mut self, context: &PassContext) {
        if self.image_pending_barriers.is_empty() && self.buffer_pending_barriers.is_empty() {
            return;
        }

        let src_stage = self.image_pending_barriers.iter().map(|barrier| barrier.src_stage)
            .chain(self.buffer_pending_barriers.iter().map(|barrier| barrier.src_stage))
            .fold(PipelineStageFlags::empty(), |acc, stage| acc | stage);

        let dst_stage = self.image_pending_barriers.iter().map(|barrier| barrier.dst_stage)
            .chain(self.buffer_pending_barriers.iter().map(|barrier| barrier.dst_stage))
            .fold(PipelineStageFlags::empty(), |acc, stage| acc | stage);

        let image_barriers = self.image_pending_barriers.iter()
            .map(|barrier| {
                ImageMemoryBarrier::default()
                    .image(barrier.image)
                    .subresource_range(barrier.subresource_range)
                    .old_layout(barrier.old_layout)
                    .new_layout(barrier.new_layout)
                    .src_access_mask(barrier.src_access)
                    .dst_access_mask(barrier.dst_access)
                    .src_queue_family_index(QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(QUEUE_FAMILY_IGNORED)
            })
            .collect::<Vec<_>>();

        let buffer_barriers = self.buffer_pending_barriers.iter()
            .map(|barrier| {
                BufferMemoryBarrier::default()
                    .buffer(barrier.buffer)
                    .offset(barrier.offset)
                    .size(barrier.size)
                    .src_access_mask(barrier.src_access)
                    .dst_access_mask(barrier.dst_access)
                    .src_queue_family_index(QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(QUEUE_FAMILY_IGNORED)
            })
            .collect::<Vec<_>>();

        context.pipeline_barrier(
            src_stage,
            dst_stage,
            DependencyFlags::empty(),
            &[],
            &buffer_barriers,
            &image_barriers,
        );

        self.image_pending_barriers.clear();
        self.buffer_pending_barriers.clear();
    }
}
