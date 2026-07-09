use ash::vk::{AccessFlags, Buffer, BufferMemoryBarrier, DependencyFlags, DeviceSize, Image, ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, MemoryBarrier, PipelineStageFlags, QUEUE_FAMILY_IGNORED};
use std::collections::HashMap;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::resource_state_tracker::pending_acceleration_structure_barrier::PendingAccelerationStructureBarrier;
use crate::render::render_graph::virtual_acceleration_structure::virtual_acceleration_structure::VirtualAccelerationStructure;
use crate::render::render_graph::resource_state_tracker::buffer_region_key::BufferRegionKey;
use crate::render::render_graph::resource_state_tracker::buffer_region_state::BufferRegionState;
use crate::render::render_graph::resource_state_tracker::buffer_state::BufferState;
use crate::render::render_graph::resource_state_tracker::image_region_key::ImageRegionKey;
use crate::render::render_graph::resource_state_tracker::image_region_state::ImageRegionState;
use crate::render::render_graph::resource_state_tracker::image_state::ImageState;
use crate::render::render_graph::resource_state_tracker::image_pending_barrier::PendingImageBarrier;
use crate::render::render_graph::resource_state_tracker::pending_buffer_barrier::PendingBufferBarrier;

pub struct ResourceStateTracker {
    image_transient_regions: Vec<ImageRegionState>,
    image_persistent_states: HashMap<Image, ImageState>,
    image_pending_barriers: Vec<PendingImageBarrier>,

    buffer_region_states: Vec<BufferRegionState>,
    buffer_pending_barriers: Vec<PendingBufferBarrier>,

    acceleration_structure_states: HashMap<VirtualAccelerationStructure, BufferState>,
    acceleration_structure_pending_barriers: Vec<PendingAccelerationStructureBarrier>,
}

impl ResourceStateTracker {
    pub fn new() -> Self {
        Self {
            image_transient_regions: Vec::new(),
            image_persistent_states: HashMap::new(),
            image_pending_barriers: Vec::new(),

            buffer_region_states: Vec::new(),
            buffer_pending_barriers: Vec::new(),

            acceleration_structure_states: HashMap::new(),
            acceleration_structure_pending_barriers: Vec::new(),
        }
    }

    pub fn register_persistent_image(
        &mut self,
        image: Image,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) {
        let write_bits = AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
            | AccessFlags::COLOR_ATTACHMENT_WRITE
            | AccessFlags::SHADER_WRITE
            | AccessFlags::TRANSFER_WRITE
            | AccessFlags::HOST_WRITE
            | AccessFlags::MEMORY_WRITE;
        let (write_access, write_stage, write_layout) = if access.intersects(write_bits) {
            (access, stage, layout)
        } else {
            (AccessFlags::empty(), PipelineStageFlags::empty(), ImageLayout::UNDEFINED)
        };
        self.image_persistent_states.insert(image, ImageState { layout, access, stage, write_access, write_stage, write_layout });
    }

    pub fn begin_frame(&mut self) {
        debug_assert!(self.image_pending_barriers.is_empty(), "Unflushed image barriers at begin_frame");
        debug_assert!(self.buffer_pending_barriers.is_empty(), "Unflushed buffer barriers at begin_frame");
        debug_assert!(self.acceleration_structure_pending_barriers.is_empty(), "Unflushed acceleration structure barriers at begin_frame");
    }

    pub fn acceleration_structure_transition(
        &mut self,
        acceleration_structure: VirtualAccelerationStructure,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) {
        let had_state = self.acceleration_structure_states.contains_key(&acceleration_structure);
        let current = self.acceleration_structure_states
            .get(&acceleration_structure)
            .copied()
            .unwrap_or(BufferState::initial());

        let write_bits = AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR
            | AccessFlags::SHADER_WRITE
            | AccessFlags::TRANSFER_WRITE
            | AccessFlags::MEMORY_WRITE;

        let both_read_only = !current.access.intersects(write_bits) && !access.intersects(write_bits);

        if had_state && both_read_only && current.access.contains(access) {
            return;
        }

        self.acceleration_structure_states.insert(acceleration_structure, BufferState { access, stage });

        self.acceleration_structure_pending_barriers.push(PendingAccelerationStructureBarrier {
            src_access: current.access,
            dst_access: access,
            src_stage: current.stage,
            dst_stage: stage,
        });
    }

    pub fn image_transition(
        &mut self,
        image: Image,
        subresource_range: ImageSubresourceRange,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) {
        let write_bits = AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
            | AccessFlags::COLOR_ATTACHMENT_WRITE
            | AccessFlags::SHADER_WRITE
            | AccessFlags::TRANSFER_WRITE
            | AccessFlags::HOST_WRITE
            | AccessFlags::MEMORY_WRITE;

        let is_write = access.intersects(write_bits);

        let redundant = |current: ImageState| {
            let both_read_only = !current.access.intersects(write_bits) && !is_write;
            current.layout == layout && both_read_only && current.access.contains(access)
        };

        let state_with_write = |write_access: AccessFlags, write_stage: PipelineStageFlags, write_layout: ImageLayout| {
            if is_write {
                ImageState { layout, access, stage, write_access: access, write_stage: stage, write_layout: layout }
            } else {
                ImageState { layout, access, stage, write_access, write_stage, write_layout }
            }
        };

        let emit_barrier = |barriers: &mut Vec<PendingImageBarrier>, current: ImageState| {
            let carry_write = current.layout != layout && current.layout == current.write_layout;
            let src_access = if carry_write { current.access | current.write_access } else { current.access };
            let src_stage = if carry_write { current.stage | current.write_stage } else { current.stage };
            barriers.push(PendingImageBarrier {
                image,
                subresource_range,
                old_layout: current.layout,
                new_layout: layout,
                src_access,
                dst_access: access,
                src_stage: if src_stage.is_empty() { PipelineStageFlags::TOP_OF_PIPE } else { src_stage },
                dst_stage: stage,
            });
        };

        if let Some(&current) = self.image_persistent_states.get(&image) {
            if redundant(current) {
                return;
            }

            self.image_persistent_states.insert(image, state_with_write(current.write_access, current.write_stage, current.write_layout));
            emit_barrier(&mut self.image_pending_barriers, current);

            return;
        }

        let mip_0 = subresource_range.base_mip_level;
        let mip_1 = mip_0 + subresource_range.level_count;
        let layer_0 = subresource_range.base_array_layer;
        let layer_1 = layer_0 + subresource_range.layer_count;

        let overlapping: Vec<usize> = self.image_transient_regions.iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.region.image == image
                    && entry.region.aspect_mask.intersects(subresource_range.aspect_mask)
                    && entry.region.base_mip_level < mip_1
                    && mip_0 < entry.region.base_mip_level + entry.region.level_count
                    && entry.region.base_array_layer < layer_1
                    && layer_0 < entry.region.base_array_layer + entry.region.layer_count
            })
            .map(|(index, _)| index)
            .collect();

        let mut carried_write_access = AccessFlags::empty();
        let mut carried_write_stage = PipelineStageFlags::empty();
        let mut carried_write_layout = ImageLayout::UNDEFINED;

        if overlapping.is_empty() {
            emit_barrier(&mut self.image_pending_barriers, ImageState::undefined());
        } else {
            if overlapping.iter().all(|&index| redundant(self.image_transient_regions[index].state)) {
                return;
            }

            let mut remainders: Vec<ImageRegionState> = Vec::new();
            for &index in &overlapping {
                let region = self.image_transient_regions[index].region;
                let state = self.image_transient_regions[index].state;

                carried_write_access |= state.write_access;
                carried_write_stage |= state.write_stage;
                if carried_write_layout == ImageLayout::UNDEFINED {
                    carried_write_layout = state.write_layout;
                }

                let region_mip_1 = region.base_mip_level + region.level_count;
                let region_layer_1 = region.base_array_layer + region.layer_count;

                let cut_mip_0 = region.base_mip_level.max(mip_0);
                let cut_mip_1 = region_mip_1.min(mip_1);
                let cut_layer_0 = region.base_array_layer.max(layer_0);
                let cut_layer_1 = region_layer_1.min(layer_1);

                if !redundant(state) {
                    let carry_write = state.layout != layout && state.layout == state.write_layout;
                    let src_access = if carry_write { state.access | state.write_access } else { state.access };
                    let src_stage = if carry_write { state.stage | state.write_stage } else { state.stage };
                    self.image_pending_barriers.push(PendingImageBarrier {
                        image,
                        subresource_range: ImageSubresourceRange {
                            aspect_mask: subresource_range.aspect_mask,
                            base_mip_level: cut_mip_0,
                            level_count: cut_mip_1 - cut_mip_0,
                            base_array_layer: cut_layer_0,
                            layer_count: cut_layer_1 - cut_layer_0,
                        },
                        old_layout: state.layout,
                        new_layout: layout,
                        src_access,
                        dst_access: access,
                        src_stage: if src_stage.is_empty() { PipelineStageFlags::TOP_OF_PIPE } else { src_stage },
                        dst_stage: stage,
                    });
                }

                if region.base_mip_level < cut_mip_0 {
                    remainders.push(ImageRegionState::sub_region(region, state, region.base_mip_level, cut_mip_0, region.base_array_layer, region_layer_1));
                }
                if cut_mip_1 < region_mip_1 {
                    remainders.push(ImageRegionState::sub_region(region, state, cut_mip_1, region_mip_1, region.base_array_layer, region_layer_1));
                }
                if region.base_array_layer < cut_layer_0 {
                    remainders.push(ImageRegionState::sub_region(region, state, cut_mip_0, cut_mip_1, region.base_array_layer, cut_layer_0));
                }
                if cut_layer_1 < region_layer_1 {
                    remainders.push(ImageRegionState::sub_region(region, state, cut_mip_0, cut_mip_1, cut_layer_1, region_layer_1));
                }
            }

            for &index in overlapping.iter().rev() {
                self.image_transient_regions.swap_remove(index);
            }

            self.image_transient_regions.extend(remainders);
        }

        self.image_transient_regions.push(ImageRegionState {
            region: ImageRegionKey {
                image,
                aspect_mask: subresource_range.aspect_mask,
                base_mip_level: mip_0,
                level_count: mip_1 - mip_0,
                base_array_layer: layer_0,
                layer_count: layer_1 - layer_0,
            },
            state: state_with_write(carried_write_access, carried_write_stage, carried_write_layout),
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
        if self.image_pending_barriers.is_empty()
            && self.buffer_pending_barriers.is_empty()
            && self.acceleration_structure_pending_barriers.is_empty()
        {
            return;
        }

        let src_stage = self.image_pending_barriers.iter().map(|barrier| barrier.src_stage)
            .chain(self.buffer_pending_barriers.iter().map(|barrier| barrier.src_stage))
            .chain(self.acceleration_structure_pending_barriers.iter().map(|barrier| barrier.src_stage))
            .fold(PipelineStageFlags::empty(), |acc, stage| acc | stage);

        let dst_stage = self.image_pending_barriers.iter().map(|barrier| barrier.dst_stage)
            .chain(self.buffer_pending_barriers.iter().map(|barrier| barrier.dst_stage))
            .chain(self.acceleration_structure_pending_barriers.iter().map(|barrier| barrier.dst_stage))
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

        let memory_barriers = self.acceleration_structure_pending_barriers.iter()
            .map(|barrier| {
                MemoryBarrier::default()
                    .src_access_mask(barrier.src_access)
                    .dst_access_mask(barrier.dst_access)
            })
            .collect::<Vec<_>>();

        context.pipeline_barrier(
            src_stage,
            dst_stage,
            DependencyFlags::empty(),
            &memory_barriers,
            &buffer_barriers,
            &image_barriers,
        );

        self.image_pending_barriers.clear();
        self.buffer_pending_barriers.clear();
        self.acceleration_structure_pending_barriers.clear();
    }
}
