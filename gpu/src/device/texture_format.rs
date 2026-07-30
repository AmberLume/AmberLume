use ash::vk::{Format, PhysicalDeviceFeatures};
use basis_universal::TranscoderBlockFormat;

#[derive(Debug, Clone, Copy)]
pub struct TextureFormat {
    pub color_srgb: Format,
    pub linear: Format,
    pub transcoder_block_format: TranscoderBlockFormat,
}

impl TextureFormat {
    pub const BC7: Self = Self {
        color_srgb: Format::BC7_SRGB_BLOCK,
        linear: Format::BC7_UNORM_BLOCK,
        transcoder_block_format: TranscoderBlockFormat::BC7,
    };

    pub const ASTC_4X4: Self = Self {
        color_srgb: Format::ASTC_4X4_SRGB_BLOCK,
        linear: Format::ASTC_4X4_UNORM_BLOCK,
        transcoder_block_format: TranscoderBlockFormat::ASTC_4x4,
    };

    pub fn pick_for_device(features: &PhysicalDeviceFeatures) -> Self {
        if features.texture_compression_bc != 0 {
            Self::BC7
        } else if features.texture_compression_astc_ldr != 0 {
            Self::ASTC_4X4
        } else {
            panic!("GPU supports neither BC nor ASTC LDR - unsupported hardware");
        }
    }
}
