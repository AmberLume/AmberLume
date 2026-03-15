use crate::render::factories::sampler::sampler_description::SamplerDescription;
use crate::render::factories::sampler::sampler_factory::SamplerFactory;
use anyhow::Result;
use ash::vk::{BorderColor, CompareOp, Filter, Sampler, SamplerAddressMode};

pub struct PersistentSamplers {
    pub depth: Sampler,
    pub linear_repeat: Sampler,
    pub linear_clamp: Sampler,
    pub shadow_mask: Sampler,
    pub shadow: Sampler,
}

impl PersistentSamplers {
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

    pub fn destroy(
        self,
        sampler_factory: &SamplerFactory,
    ) -> Result<()> {
        sampler_factory.destroy_sampler(self.depth)?;
        sampler_factory.destroy_sampler(self.linear_repeat)?;
        sampler_factory.destroy_sampler(self.linear_clamp)?;
        sampler_factory.destroy_sampler(self.shadow_mask)?;
        sampler_factory.destroy_sampler(self.shadow)?;

        Ok(())
    }
}
