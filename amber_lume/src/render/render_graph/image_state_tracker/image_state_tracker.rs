use ash::vk::{AccessFlags, Image, ImageLayout, ImageSubresourceRange, PipelineStageFlags};
use std::collections::HashMap;
use crate::render::pass::pass_context::PassContext;

pub struct ImageStateTracker {
    transient_states: HashMap<Image, ImageState>,
    persistent_states: HashMap<Image, ImageState>,
}

#[derive(Copy, Clone)]
struct ImageState {
    layout: ImageLayout,
    access: AccessFlags,
    stage: PipelineStageFlags,
}

impl ImageState {
    fn undefined() -> Self {
        Self {
            layout: ImageLayout::UNDEFINED,
            access: AccessFlags::empty(),
            stage: PipelineStageFlags::TOP_OF_PIPE,
        }
    }
}

impl ImageStateTracker {
    pub fn new() -> Self {
        Self {
            transient_states: HashMap::new(),
            persistent_states: HashMap::new(),
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
    }

    pub fn transition(
        &mut self,
        context: &PassContext,
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

        if current.layout == layout && current.access == access && current.stage == stage {
            return;
        }

        let state = ImageState { layout, access, stage };

        if self.persistent_states.contains_key(&image) {
            self.persistent_states.insert(image, state);
        } else {
            self.transient_states.insert(image, state);
        }

        context.transition_image_layout(
            image,
            subresource_range,
            current.layout,
            layout,
            current.access,
            access,
            current.stage,
            stage,
        )
    }
}
