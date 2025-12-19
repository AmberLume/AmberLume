use crate::render::vulkan::buffer::resource_context::ResourceContext;
use crate::render::vulkan::device_context::DeviceContext;
use crate::resources::common::resource_provider::ResourceProvider;
use crate::resources::descriptor_set_layout::descriptor_set_layout_backend::DescriptorSetLayoutBackend;
use crate::resources::index::resource_index::ResourceIndex;
use crate::resources::model::model_backend::ModelBackend;
use crate::resources::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::pipeline_layout::pipeline_layout_backend::PipelineLayoutBackend;
use crate::resources::providers::io_provider::IOProvider;
use crate::resources::shader::shader_backend::ShaderBackend;
use anyhow::Result;
use ash::vk::PipelineCache;
use std::sync::Arc;

pub struct ResourceHub {
    shader_provider: Arc<ResourceProvider<ShaderBackend>>,
    descriptor_set_layout_provider: Arc<ResourceProvider<DescriptorSetLayoutBackend>>,
    pipeline_layout_provider: Arc<ResourceProvider<PipelineLayoutBackend>>,
    pipeline_provider: Arc<ResourceProvider<PipelineBackend>>,
    model_provider: Arc<ResourceProvider<ModelBackend>>,
}

impl ResourceHub {
    pub fn new(
        device_context: &DeviceContext,
        resource_context: &mut ResourceContext,
        io_provider: Arc<dyn IOProvider>,
    ) -> Result<Self> {
        let resource_index = {
            let resource_index = ResourceIndex::new(io_provider.clone())?;

            Arc::new(resource_index)
        };

        let shader_provider = {
            let shader_backend = ShaderBackend::new(&device_context, resource_index.clone());

            ResourceProvider::from(shader_backend)
        };

        let descriptor_set_layout_provider = {
            let descriptor_set_layout_backend = DescriptorSetLayoutBackend::new(&device_context);

            ResourceProvider::from(descriptor_set_layout_backend)
        };

        let pipeline_layout_provider = {
            let shader_backend =
                PipelineLayoutBackend::new(&device_context, descriptor_set_layout_provider.clone());

            ResourceProvider::from(shader_backend)
        };

        let pipeline_provider = {
            let pipeline_backend = PipelineBackend::new(
                &device_context,
                shader_provider.clone(),
                pipeline_layout_provider.clone(),
                PipelineCache::null(),
            );

            ResourceProvider::from(pipeline_backend)
        };

        let model_provider = {
            let model_backend =
                ModelBackend::new(&device_context, resource_index.clone(), resource_context);

            ResourceProvider::from(model_backend)
        };

        Ok(Self {
            shader_provider,
            descriptor_set_layout_provider,
            pipeline_layout_provider,
            pipeline_provider,
            model_provider,
        })
    }

    pub fn get_pipeline_provider(&self) -> Arc<ResourceProvider<PipelineBackend>> {
        self.pipeline_provider.clone()
    }

    pub fn get_model_provider(&self) -> Arc<ResourceProvider<ModelBackend>> {
        self.model_provider.clone()
    }

    pub fn destroy(&self) -> Result<()> {
        self.model_provider.destroy()?;
        self.pipeline_provider.destroy()?;
        self.pipeline_layout_provider.destroy()?;
        self.descriptor_set_layout_provider.destroy()?;
        self.shader_provider.destroy()?;

        Ok(())
    }
}
