use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct AnimationData {
    pub name: String,
    pub duration: f32,
    pub fps: f32,
    pub bone_count: u32,
    pub frame_count: u32,
    pub keyframes: Vec<AnimationKeyframe>,
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AnimationKeyframe {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}
