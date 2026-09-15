pub enum AnimationTransitionSource {
    Any,
    States {
        indices: Vec<u16>,
    },
}

impl AnimationTransitionSource {
    pub fn matches(&self, state: u16) -> bool {
        match self {
            AnimationTransitionSource::Any => true,
            AnimationTransitionSource::States { indices } => indices.contains(&state),
        }
    }
}
