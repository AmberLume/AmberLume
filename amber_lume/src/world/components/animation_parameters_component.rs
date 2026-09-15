use shipyard::Component;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AnimationFloatParameter {
    Speed,
    VerticalSpeed,

    Count,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AnimationFlagParameter {
    Grounded,

    Count,
}

#[derive(Component)]
pub struct AnimationParametersComponent {
    pub floats: [f32; AnimationFloatParameter::Count as usize],
    pub flags: [bool; AnimationFlagParameter::Count as usize],
}

impl AnimationParametersComponent {
    pub const INITIAL: Self = Self {
        floats: [0.0; AnimationFloatParameter::Count as usize],
        flags: [false; AnimationFlagParameter::Count as usize],
    };
}
