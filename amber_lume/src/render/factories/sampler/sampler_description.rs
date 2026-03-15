use ash::vk::{BorderColor, CompareOp, Filter, SamplerAddressMode, SamplerMipmapMode};

pub struct SamplerDescription {
    pub mag_filter: Filter,
    pub min_filter: Filter,

    pub address_mode_u: SamplerAddressMode,
    pub address_mode_v: SamplerAddressMode,
    pub address_mode_w: SamplerAddressMode,

    pub anisotropy_enable: bool,
    pub max_anisotropy: f32,

    pub mipmap_mode: SamplerMipmapMode,
    pub mip_lod_bias: f32,

    pub min_lod: f32,
    pub max_lod: f32,

    pub border_color: BorderColor,

    pub compare_enable: bool,
    pub compare_op: CompareOp,
}

impl Default for SamplerDescription {
    fn default() -> Self {
        Self {
            mag_filter: Filter::LINEAR,
            min_filter: Filter::LINEAR,

            address_mode_u: SamplerAddressMode::REPEAT,
            address_mode_v: SamplerAddressMode::REPEAT,
            address_mode_w: SamplerAddressMode::REPEAT,

            anisotropy_enable: true,
            max_anisotropy: 16.0,

            mipmap_mode: SamplerMipmapMode::LINEAR,
            mip_lod_bias: 0.0,

            min_lod: 0.0,
            max_lod: 12.0,

            border_color: BorderColor::INT_OPAQUE_BLACK,

            compare_enable: false,
            compare_op: CompareOp::NEVER,
        }
    }
}