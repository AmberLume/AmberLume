use std::sync::Arc;
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::common::resource_provider::ResourceId;
use anyhow::Result;
use ash::Device;
use ash::vk::{Sampler, SamplerCreateInfo};
use crate::render::vulkan::debug_utils::DebugUtils;
use crate::render::vulkan::device_context::DeviceContext;
use crate::resources::sampler::sampler_config::SamplerConfig;

pub struct SamplerBackend {
    device: Device,
    debug_utils: Arc<DebugUtils>,
}

impl SamplerBackend {
    pub fn new(
        device_context: &DeviceContext,
    ) -> Self {
        Self {
            device: device_context.device.clone(),
            debug_utils: device_context.debug_utils.clone(),
        }
    }
}

pub struct SamplerDependencies;

impl ResourceBackend for SamplerBackend {
    type Config = SamplerConfig;
    type Dependencies = SamplerDependencies;
    type Output = Sampler;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn collect_dependencies(&self, _config: &Self::Config) -> Self::Dependencies {
        Self::Dependencies { }
    }

    fn create(
        &self,
        _id: &ResourceId,
        config: Self::Config,
        _dependencies: Self::Dependencies,
    ) -> Result<Self::Output> {
        let sampler_info = SamplerCreateInfo::default()
            .mag_filter(config.mag_filter)
            .min_filter(config.min_filter)
            .address_mode_u(config.address_mode_u)
            .address_mode_v(config.address_mode_v)
            .address_mode_w(config.address_mode_w)
            .anisotropy_enable(config.anisotropy_enable)
            .max_anisotropy(config.max_anisotropy)
            .mipmap_mode(config.mipmap_mode)
            .mip_lod_bias(config.mip_lod_bias)
            .min_lod(config.min_lod)
            .max_lod(config.max_lod)
            .border_color(config.border_color)
            .compare_enable(config.compare_enable)
            .compare_op(config.compare_op)
            .unnormalized_coordinates(false);

        let sampler = unsafe { self.device.create_sampler(&sampler_info, None).unwrap() };

        self.debug_utils.label(sampler, &format!("{:?}", config));

        Ok(sampler)
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        unsafe { self.device.destroy_sampler(resource, None) }

        Ok(())
    }

    fn destroy(&mut self) -> Result<()> {
        Ok(())
    }
}
