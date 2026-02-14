use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::factories::descriptor_set::descriptor_set_layout_factory::DescriptorSetLayoutFactory;
use crate::render::vulkan::factories::image::managed_image_factory::ManagedImageFactory;

pub struct ResourceFactories {
    pub managed_buffer_factory: ManagedBufferFactory,
    pub managed_image_factory: ManagedImageFactory,
    pub descriptor_set_layout_factory: DescriptorSetLayoutFactory,
}

impl ResourceFactories {
    pub fn create(
        device_context: &DeviceContext,
    ) -> Self {
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

        Self {
            managed_buffer_factory,
            managed_image_factory,
            descriptor_set_layout_factory,
        }
    }
}
