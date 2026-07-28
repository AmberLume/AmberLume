use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AlphaMode {
    Opaque,
    Mask,
    Blend,
}

impl AlphaMode {
    pub const DEFAULT_CUTOFF: f32 = 0.5;
}

impl From<&ArchivedAlphaMode> for AlphaMode {
    fn from(value: &ArchivedAlphaMode) -> Self {
        match value {
            ArchivedAlphaMode::Opaque => Self::Opaque,
            ArchivedAlphaMode::Mask => Self::Mask,
            ArchivedAlphaMode::Blend => Self::Blend,
        }
    }
}
