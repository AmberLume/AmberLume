use crate::render::factories::sampler::sampler_description::SamplerDescription;
use crate::render::factories::sampler::sampler_factory::SamplerFactory;
use anyhow::Result;
use ash::vk::{BorderColor, CompareOp, Filter, Sampler, SamplerAddressMode};

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum SamplerType {
    Depth,

    LinearRepeat,
    LinearClamp,

    Shadow,
    ShadowMask,
}

pub struct SamplerRegistry {
    depth: Sampler,

    linear_repeat: Sampler,
    linear_clamp: Sampler,

    shadow: Sampler,
    shadow_mask: Sampler,
}

impl SamplerRegistry {
    pub fn create(sampler_factory: &SamplerFactory) -> Result<Self> {
        let depth = sampler_factory.create_sampler(
            "depth",
            SamplerDescription::default(),
        )?;

        let linear_repeat = sampler_factory.create_sampler(
            "linear_repeat",
            SamplerDescription::default(),
        )?;

        let linear_clamp = sampler_factory.create_sampler(
            "linear_clamp",
            SamplerDescription {
                address_mode_u: SamplerAddressMode::CLAMP_TO_EDGE,
                address_mode_v: SamplerAddressMode::CLAMP_TO_EDGE,
                address_mode_w: SamplerAddressMode::CLAMP_TO_EDGE,

                ..SamplerDescription::default()
            },
        )?;

        let shadow_mask = sampler_factory.create_sampler(
            "shadow_mask",
            SamplerDescription {
                mag_filter: Filter::NEAREST,
                min_filter: Filter::NEAREST,

                address_mode_u: SamplerAddressMode::CLAMP_TO_BORDER,
                address_mode_v: SamplerAddressMode::CLAMP_TO_BORDER,
                address_mode_w: SamplerAddressMode::CLAMP_TO_BORDER,

                anisotropy_enable: false,

                max_lod: 0.0,
                border_color: BorderColor::FLOAT_OPAQUE_WHITE,

                compare_enable: false,
                compare_op: CompareOp::LESS_OR_EQUAL,

                ..SamplerDescription::default()
            },
        )?;

        let shadow = sampler_factory.create_sampler(
            "shadow",
            SamplerDescription {
                address_mode_u: SamplerAddressMode::CLAMP_TO_BORDER,
                address_mode_v: SamplerAddressMode::CLAMP_TO_BORDER,
                address_mode_w: SamplerAddressMode::CLAMP_TO_BORDER,

                anisotropy_enable: false,

                max_lod: 0.0,
                border_color: BorderColor::FLOAT_OPAQUE_WHITE,

                compare_enable: true,
                compare_op: CompareOp::LESS,

                ..SamplerDescription::default()
            },
        )?;

        Ok(Self {
            depth,
            linear_repeat,
            linear_clamp,
            shadow_mask,
            shadow,
        })
    }

    pub fn get(&self, sampler_type: SamplerType) -> Sampler {
        match sampler_type {
            SamplerType::Depth => self.depth,
            SamplerType::LinearRepeat => self.linear_repeat,
            SamplerType::LinearClamp => self.linear_clamp,
            SamplerType::Shadow => self.shadow,
            SamplerType::ShadowMask => self.shadow_mask,
        }
    }

    pub fn destroy(&self, sampler_factory: &SamplerFactory) {
        sampler_factory.destroy_sampler(self.depth);
        sampler_factory.destroy_sampler(self.linear_repeat);
        sampler_factory.destroy_sampler(self.linear_clamp);
        sampler_factory.destroy_sampler(self.shadow_mask);
        sampler_factory.destroy_sampler(self.shadow);
    }
}
