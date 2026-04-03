use ash::vk::{AccessFlags, DependencyFlags, Image, ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, PipelineStageFlags, QUEUE_FAMILY_IGNORED};
use std::collections::HashMap;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::image_state_tracker::image_state::ImageState;
use crate::render::render_graph::image_state_tracker::pending_barrier::PendingBarrier;

pub struct ImageStateTracker {
    transient_states: HashMap<Image, ImageState>,
    persistent_states: HashMap<Image, ImageState>,
    pending: Vec<PendingBarrier>,
}

impl ImageStateTracker {
    pub fn new() -> Self {
        Self {
            transient_states: HashMap::new(),
            persistent_states: HashMap::new(),
            pending: Vec::new(),
        }
    }

    pub fn register_persistent(
        &mut self,
        image: Image,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) {
        self.persistent_states.insert(image, ImageState { layout, access, stage });
    }

    pub fn begin_frame(&mut self) {
        self.transient_states.clear();

        debug_assert!(self.pending.is_empty(), "Unflushed barriers at begin_frame");
    }

    pub fn transition(
        &mut self,
        image: Image,
        subresource_range: ImageSubresourceRange,
        layout: ImageLayout,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) {
        let current = self.transient_states.get(&image)
            .or_else(|| self.persistent_states.get(&image))
            .copied()
            .unwrap_or_else(ImageState::undefined);

        if current.layout == layout && current.access == access {
            return;
        }

        let state = ImageState { layout, access, stage };

        if self.persistent_states.contains_key(&image) {
            self.persistent_states.insert(image, state);
        } else {
            self.transient_states.insert(image, state);
        }

        self.pending.push(PendingBarrier {
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

    pub fn flush(&mut self, context: &PassContext) {
        if self.pending.is_empty() {
            return;
        }

        let src_stage = self.pending.iter()
            .fold(PipelineStageFlags::empty(), |acc, p| acc | p.src_stage);
        let dst_stage = self.pending.iter()
            .fold(PipelineStageFlags::empty(), |acc, p| acc | p.dst_stage);

        let barriers = self.pending.iter()
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

        context.pipeline_barrier(
            src_stage,
            dst_stage,
            DependencyFlags::empty(),
            &[],
            &[],
            &barriers,
        );

        self.pending.clear();
    }
}
