use std::sync::Arc;
use anyhow::Result;
use ash::Device;
use ash::vk::{Sampler, SamplerCreateInfo};
use crate::render::vulkan::debug_utils::DebugUtils;
use crate::render::vulkan::factories::sampler::sampler_description::SamplerDescription;

pub struct SamplerFactory {
    device: Device,
    debug_utils: Arc<DebugUtils>,
}

impl SamplerFactory {
    pub fn create(
        device: Device,
        debug_utils: Arc<DebugUtils>,
    ) -> Self {
        Self {
            device,
            debug_utils,
        }
    }

    pub fn create_sampler(
        &self, 
        label: &str,
        description: SamplerDescription,
    ) -> Result<Sampler> {
        let sampler_info = SamplerCreateInfo::default()
            .mag_filter(description.mag_filter)
            .min_filter(description.min_filter)
            .address_mode_u(description.address_mode_u)
            .address_mode_v(description.address_mode_v)
            .address_mode_w(description.address_mode_w)
            .anisotropy_enable(description.anisotropy_enable)
            .max_anisotropy(description.max_anisotropy)
            .mipmap_mode(description.mipmap_mode)
            .mip_lod_bias(description.mip_lod_bias)
            .min_lod(description.min_lod)
            .max_lod(description.max_lod)
            .border_color(description.border_color)
            .compare_enable(description.compare_enable)
            .compare_op(description.compare_op)
            .unnormalized_coordinates(false);

        let sampler = unsafe { self.device.create_sampler(&sampler_info, None)? };

        self.debug_utils.label(sampler, &format!("{:?}", label));

        Ok(sampler)
    }

    pub fn destroy_sampler(&self, sampler: Sampler) -> Result<()> {
        unsafe { self.device.destroy_sampler(sampler, None) }

        Ok(())
    }
}
