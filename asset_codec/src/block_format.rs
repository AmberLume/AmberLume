use basis_universal::TranscoderBlockFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockFormat {
    Bc7,
    Astc4x4,
}

impl BlockFormat {
    pub(crate) fn transcoder_block_format(&self) -> TranscoderBlockFormat {
        match self {
            Self::Bc7 => TranscoderBlockFormat::BC7,
            Self::Astc4x4 => TranscoderBlockFormat::ASTC_4x4,
        }
    }
}
