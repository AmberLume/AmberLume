use crate::render::device::device_context::DeviceContext;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::descriptor_set::descriptor_set_factory::DescriptorSetFactory;
use crate::render::factories::descriptor_set_layout::descriptor_set_layout_factory::DescriptorSetLayoutFactory;
use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::render::factories::pipeline_layout::pipeline_layout_factory::PipelineLayoutFactory;
use crate::render::factories::sampler::sampler_factory::SamplerFactory;
use anyhow::Result;
use crate::render::factories::query_pool::query_pool_factory::QueryPoolFactory;

pub struct ResourceFactories {
    pub sampler_factory: SamplerFactory,
    pub buffer_factory: ManagedBufferFactory,
    pub managed_image_factory: ManagedImageFactory,
    pub descriptor_set_layout_factory: DescriptorSetLayoutFactory,
    pub descriptor_set_factory: DescriptorSetFactory,
    pub query_pool_factory: QueryPoolFactory,
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
            device_context.physical_device_info.supports_ray_tracing(),
        )?;
        
        let query_pool_factory = QueryPoolFactory::create(
            device_context.device.clone(),
            device_context.debug_utils.clone(),
        );
        
        let pipeline_layout_factory = PipelineLayoutFactory::create(
            device_context.device.clone(),
            device_context.debug_utils.clone(),
        );

        Ok(Self {
            sampler_factory,
            buffer_factory: managed_buffer_factory,
            managed_image_factory,
            descriptor_set_layout_factory,
            descriptor_set_factory,
            query_pool_factory,
            pipeline_layout_factory,
        })
    }
    
    pub fn destroy(&self) {
        self.descriptor_set_factory.destroy();
    }
}
