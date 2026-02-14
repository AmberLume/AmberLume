use ash::vk::{BorderColor, CompareOp, Filter, SamplerAddressMode, SamplerMipmapMode};
use crate::resources::common::resource_backend::ResourceKey;
use crate::resources::utils::hasher::hasher::Hasher;

#[derive(Clone, Debug)]
pub struct SamplerConfig {
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

impl SamplerConfig {
    pub fn default() -> SamplerConfig {
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

    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_i32(self.mag_filter.as_raw());
        hasher.hash_i32(self.min_filter.as_raw());

        hasher.hash_i32(self.address_mode_u.as_raw());
        hasher.hash_i32(self.address_mode_v.as_raw());
        hasher.hash_i32(self.address_mode_w.as_raw());

        hasher.hash_bool(self.anisotropy_enable);
        hasher.hash_f32(self.max_anisotropy);

        hasher.hash_i32(self.mipmap_mode.as_raw());
        hasher.hash_f32(self.mip_lod_bias);

        hasher.hash_f32(self.min_lod);
        hasher.hash_f32(self.max_lod);

        hasher.hash_i32(self.border_color.as_raw());

        hasher.hash_bool(self.compare_enable);
        hasher.hash_i32(self.compare_op.as_raw());

        hasher.finalize()
    }
}
