use std::any::type_name;
use std::sync::Arc;
use crate::animation_state::AnimationState;
use crate::play_mode::PlayMode;
use resource_residency::ResRef;

pub struct AnimationMappingEntry {
    pub handle: Arc<ResRef>,

    pub duration: f32,
    pub speed: f32,

    pub mode: PlayMode,
}

pub struct AnimationMapping {
    pub entries: Vec<AnimationMappingEntry>,
}

impl AnimationMapping {
    pub fn new<S: AnimationState>(entries: Vec<AnimationMappingEntry>) -> Self {
        assert_eq!(
            entries.len(),
            S::count() as usize,
            "AnimationMapping: got {} entries, expected {} for {}",
            entries.len(),
            S::count(),
            type_name::<S>(),
        );

        Self { entries }
    }
}