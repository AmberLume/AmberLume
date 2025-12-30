use std::sync::Arc;
use crate::render::vulkan::device_context::DeviceContext;
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use anyhow::Result;
use ash::Device;
use ash::vk::{DescriptorPool, DescriptorPoolCreateFlags, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorType};
use tracing::info;
use crate::render::vulkan::debug_utils::DebugUtils;
use crate::resources::common::resource_provider::{ResourceId, ResourceProvider};
use crate::resources::descriptor_set::descriptor_set_config::DescriptorSetConfig;
use crate::resources::descriptor_set_layout::descriptor_set_layout_backend::DescriptorSetLayoutBackend;

pub struct DescriptorSetBackend {
    device: Device,
    debug_utils: Arc<DebugUtils>,

    descriptor_set_layout_provider: Arc<ResourceProvider<DescriptorSetLayoutBackend>>,
    
    descriptor_pool: DescriptorPool,
}

impl DescriptorSetBackend {
    pub fn new(
        device_context: &DeviceContext,
        descriptor_set_layout_provider: Arc<ResourceProvider<DescriptorSetLayoutBackend>>,
    ) -> Result<Self> {
        let device = &device_context.device;

        let pool_sizes = [DescriptorPoolSize {
            ty: DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 4096,
        }];

        let pool_info = DescriptorPoolCreateInfo::default()
            .max_sets(1)
            .pool_sizes(&pool_sizes)
            .flags(DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);

        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        Ok(Self {
            device: device.clone(),
            debug_utils: device_context.debug_utils.clone(),

            descriptor_set_layout_provider,
            
            descriptor_pool,
        })
    }
}

pub struct DescriptorSetDependencies {
    pub descriptor_set_layout: DescriptorSetLayout,
}

impl ResourceBackend for DescriptorSetBackend {
    type Config = DescriptorSetConfig;
    type Dependencies = DescriptorSetDependencies;
    type Output = DescriptorSet;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn collect_dependencies(&self, config: &Self::Config) -> Self::Dependencies {
        let descriptor_set_layout = self
            .descriptor_set_layout_provider
            .get_now(&config.descriptor_set_layout_config);
        
        Self::Dependencies {
            descriptor_set_layout: *descriptor_set_layout,
        }
    }

    fn create(
        &self,
        _id: &ResourceId,
        config: Self::Config,
        dependencies: Self::Dependencies,
    ) -> Result<Self::Output> {
        let layouts = [dependencies.descriptor_set_layout];
        let descriptor_set_allocation_info = DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_set = unsafe { self.device.allocate_descriptor_sets(&descriptor_set_allocation_info)?[0] };
        
        self.debug_utils.label(descriptor_set, &format!("{:?}", config));
        
        Ok(descriptor_set)
    }

    fn destroy_resource(&self, _resource: Self::Output) -> Result<()> {
        info!("DescriptorSet destroyed");

        Ok(())
    }

    fn destroy(&mut self) -> Result<()> {
        unsafe { self.device.destroy_descriptor_pool(self.descriptor_pool, None) }

        info!("DescriptorSet backend destroyed");
        
        Ok(())
    }
}
