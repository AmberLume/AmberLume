use std::ops::Range;

pub struct ShadowLayout {
    pub shadow_cascades: Vec<Range<f32>>,
}

impl ShadowLayout {
    pub fn create() -> Self {
        Self {
            shadow_cascades: vec![0.0..8.0, 7.0..16.0, 15.0..32.0, 31.0..64.0],
        }
    }
}
