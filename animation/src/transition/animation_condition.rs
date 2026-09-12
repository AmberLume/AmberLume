use crate::parameters::animation_parameters::AnimationParameters;

pub enum AnimationCondition {
    Greater { parameter: u16, threshold: f32 },
    Less { parameter: u16, threshold: f32 },
    Equals { flag: u16, value: bool },
    Finished,
}

impl AnimationCondition {
    pub fn holds(&self, parameters: &AnimationParameters, finished: bool) -> bool {
        match self {
            AnimationCondition::Greater {
                parameter,
                threshold,
            } => parameters.floats[*parameter as usize] > *threshold,
            AnimationCondition::Less {
                parameter,
                threshold,
            } => parameters.floats[*parameter as usize] < *threshold,
            AnimationCondition::Equals { flag, value } => {
                parameters.flags[*flag as usize] == *value
            }
            AnimationCondition::Finished => finished,
        }
    }
}
