use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::factories::descriptor_set::descriptor_set_factory::DescriptorSetFactory;
use crate::render::vulkan::factories::descriptor_set_layout::descriptor_set_layout_factory::DescriptorSetLayoutFactory;
use crate::render::vulkan::factories::image::managed_image_factory::ManagedImageFactory;
use crate::render::vulkan::factories::pipeline_layout::pipeline_layout_factory::PipelineLayoutFactory;
use crate::render::vulkan::factories::sampler::sampler_factory::SamplerFactory;
use anyhow::Result;

pub struct ResourceFactories {
    pub sampler_factory: SamplerFactory,
    pub managed_buffer_factory: ManagedBufferFactory,
    pub managed_image_factory: ManagedImageFactory,
    pub descriptor_set_layout_factory: DescriptorSetLayoutFactory,
    pub descriptor_set_factory: DescriptorSetFactory,
    pub pipeline_layout_factory: PipelineLayoutFactory,
}

impl ResourceFactories {
    pub fn create(
        device_context: &DeviceContext,
    ) -> Result<Self> {
        let sampler_factory = SamplerFactory::create(
            device_context.device.clone(),
            device_context.debug_utils.clone(),
        );
        
        let managed_buffer_factory = ManagedBufferFactory::create(
            device_context.device.clone(),
            device_context.allocator.clone(),
            device_context.debug_utils.clone(),
        );
        
        let managed_image_factory = ManagedImageFactory::new(
            device_context.device.clone(),
            device_context.allocator.clone(),
            device_context.debug_utils.clone(),
        );

        let descriptor_set_layout_factory = DescriptorSetLayoutFactory::create(
            device_context.device.clone(),
            device_context.debug_utils.clone(),
        );

        let descriptor_set_factory = DescriptorSetFactory::create(
            device_context.device.clone(),
            device_context.debug_utils.clone(),
        )?;
        
        let pipeline_layout_factory = PipelineLayoutFactory::create(
            device_context.device.clone(),
            device_context.debug_utils.clone(),
        );

        Ok(Self {
            sampler_factory,
            managed_buffer_factory,
            managed_image_factory,
            descriptor_set_layout_factory,
            descriptor_set_factory,
            pipeline_layout_factory,
        })
    }
    
    pub fn destroy(
        self, 
    ) -> Result<()> {
        self.descriptor_set_factory.destroy()?;
        
        Ok(())
    }
}
