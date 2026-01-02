use crate::platform_providers::io_provider::IOProvider;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::render::vulkan::device_context::DeviceContext;
use crate::resources::common::resource_provider::ResourceProvider;
use crate::resources::descriptor_set_layout::descriptor_set_layout_backend::DescriptorSetLayoutBackend;
use crate::resources::index::resource_index::ResourceIndex;
use crate::resources::model::model_backend::ModelBackend;
use crate::resources::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::pipeline_layout::pipeline_layout_backend::PipelineLayoutBackend;
use crate::resources::shader::shader_backend::ShaderBackend;
use anyhow::Result;
use ash::vk::PipelineCache;
use std::sync::Arc;
use crate::resources::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::descriptor_set::descriptor_set_backend::DescriptorSetBackend;
use crate::resources::image::image_backend::ImageBackend;
use crate::resources::material::material_backend::MaterialBackend;
use crate::resources::sampler::sampler_backend::SamplerBackend;
use crate::resources::scene_loader::scene_loader::SceneLoader;

pub struct ResourceHub {
    pub scene_loader: Arc<SceneLoader>,

    shader_provider: Arc<ResourceProvider<ShaderBackend>>,
    descriptor_set_layout_provider: Arc<ResourceProvider<DescriptorSetLayoutBackend>>,
    pipeline_layout_provider: Arc<ResourceProvider<PipelineLayoutBackend>>,
    pipeline_provider: Arc<ResourceProvider<PipelineBackend>>,
    compute_pipeline_provider: Arc<ResourceProvider<ComputePipelineBackend>>,
    descriptor_set_provider: Arc<ResourceProvider<DescriptorSetBackend>>,
    sampler_provider: Arc<ResourceProvider<SamplerBackend>>,
    image_provider: Arc<ResourceProvider<ImageBackend>>,
    material_provider: Arc<ResourceProvider<MaterialBackend>>,
    model_provider: Arc<ResourceProvider<ModelBackend>>,
}

impl ResourceHub {
    pub fn create(
        device_context: &mut DeviceContext,
        resource_context: &mut ResourceContext,
        io_provider: Arc<dyn IOProvider>,
    ) -> Result<Self> {
        let resource_index = {
            let resource_index = ResourceIndex::new(io_provider.clone())?;

            Arc::new(resource_index)
        };

        let scene_loader = {
            let scene_loader = SceneLoader::create(resource_index.clone());

            Arc::new(scene_loader)
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

        let compute_pipeline_provider = {
            let compute_pipeline = ComputePipelineBackend::new(
                &device_context,
                shader_provider.clone(),
                pipeline_layout_provider.clone(),
                PipelineCache::null(),
            );

            ResourceProvider::from(compute_pipeline)
        };

        let descriptor_set_provider = {
            let descriptor_set_backend = DescriptorSetBackend::new(
                &device_context,
                descriptor_set_layout_provider.clone(),
            )?;
            
            ResourceProvider::from(descriptor_set_backend)
        };
        
        let sampler_provider = {
            let sampler_backend = SamplerBackend::new(&device_context);

            ResourceProvider::from(sampler_backend)
        };
        
        let image_provider = {
            let image_backend = ImageBackend::new(
                &device_context, 
                &resource_context,
                sampler_provider.clone(),
                descriptor_set_provider.clone(),
                resource_index.clone(),
            );
            
            ResourceProvider::from(image_backend)
        };
        
        let material_provider = {
            let material_backend = MaterialBackend::new(
                &resource_context,
                image_provider.clone(),
                resource_index.clone(),
            );

            ResourceProvider::from(material_backend)
        };

        let model_provider = {
            let model_backend = ModelBackend::new(
                resource_context,
                resource_index.clone(),
                material_provider.clone(),
            );

            ResourceProvider::from(model_backend)
        };

        Ok(Self {
            scene_loader,

            shader_provider,
            descriptor_set_layout_provider,
            pipeline_layout_provider,
            pipeline_provider,
            compute_pipeline_provider,
            descriptor_set_provider,
            sampler_provider,
            image_provider,
            material_provider,
            model_provider,
        })
    }

    pub fn get_pipeline_provider(&self) -> Arc<ResourceProvider<PipelineBackend>> {
        self.pipeline_provider.clone()
    }

    pub fn get_descriptor_set_provider(&self) -> Arc<ResourceProvider<DescriptorSetBackend>> {
        self.descriptor_set_provider.clone()
    }

    pub fn get_compute_pipeline_provider(&self) -> Arc<ResourceProvider<ComputePipelineBackend>> {
        self.compute_pipeline_provider.clone()
    }

    pub fn get_pipeline_layout_provider(&self) -> Arc<ResourceProvider<PipelineLayoutBackend>> {
        self.pipeline_layout_provider.clone()
    }

    pub fn get_model_provider(&self) -> Arc<ResourceProvider<ModelBackend>> {
        self.model_provider.clone()
    }

    pub fn destroy(self) -> Result<()> {
        self.model_provider.destroy()?;
        self.material_provider.destroy()?;
        self.image_provider.destroy()?;
        self.sampler_provider.destroy()?;
        self.descriptor_set_provider.destroy()?;
        self.compute_pipeline_provider.destroy()?;
        self.pipeline_provider.destroy()?;
        self.pipeline_layout_provider.destroy()?;
        self.descriptor_set_layout_provider.destroy()?;
        self.shader_provider.destroy()?;

        Ok(())
    }
}
