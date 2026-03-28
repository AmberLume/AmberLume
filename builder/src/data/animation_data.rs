use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct AnimationData {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<BoneChannel>,
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BoneChannel {
    pub positions: Vec<(f32, [f32; 3])>,
    pub rotations: Vec<(f32, [f32; 4])>,
    pub scales: Vec<(f32, [f32; 3])>,
}

impl Default for BoneChannel {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            rotations: Vec::new(),
            scales: Vec::new(),
        }
    }
}
