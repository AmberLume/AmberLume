use crate::platform_providers::io_provider::IOProvider;
use crate::render::resources::resource_context::ResourceContext;
use crate::render::device::device_context::DeviceContext;
use crate::resources::alpaca_resource_reader::alpaca_resource_reader::AlpacaResourceReader;
use crate::resources::dynamic::mesh::mesh_backend::MeshBackend;
use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use ash::vk::{PipelineCache};
use crate::render::factories::descriptor_set_layout::descriptor_set_layout_factory::DescriptorSetLayoutFactory;
use crate::render::factories::pipeline_layout::pipeline_layout_factory::PipelineLayoutFactory;
use crate::render::factories::sampler::sampler_factory::SamplerFactory;
use crate::resources::dynamic::image::image_backend::ImageBackend;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::descriptor_index_managers::IndexManagers;
use crate::resources::descriptor_set_manager::DescriptorSetManager;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::dynamic::material::material_backend::MaterialBackend;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::skeleton::skeleton_backend::SkeletonBackend;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::resources::pipeline_layout_registry::PipelineLayoutRegistry;
use crate::resources::resource_factories::ResourceFactories;
use crate::resources::scene_loader::scene_loader::SceneLoader;

pub struct ResourceHub {
    pub scene_loader: Arc<SceneLoader>,

    pub alpaca_resource_reader: Arc<AlpacaResourceReader>,

    pub descriptor_set_manager: Arc<DescriptorSetManager>,
    pub pipeline_layout_registry: Arc<PipelineLayoutRegistry>,

    pub image_provider: Arc<ResourceProvider<ImageBackend>>,
    skeletons_provider: Arc<ResourceProvider<SkeletonBackend>>,
    material_provider: Arc<ResourceProvider<MaterialBackend>>,
    mesh_provider: Arc<ResourceProvider<MeshBackend>>,
    pipeline_provider: Arc<ResourceProvider<PipelineBackend>>,
    compute_pipeline_provider: Arc<ResourceProvider<ComputePipelineBackend>>,
}

impl ResourceHub {
    pub fn create(
        device_context: &DeviceContext,
        resource_context: &mut ResourceContext,
        descriptor_set_manager: Arc<DescriptorSetManager>,
        descriptor_index_managers: Arc<IndexManagers>,
        frame_counter: Arc<AtomicU64>,
        resource_factories: Arc<ResourceFactories>,
        persistent_resources: Arc<PersistentResources>,
        io_provider: Arc<dyn IOProvider>,
    ) -> Result<Self> {
        let alpaca_resource_reader = Arc::new(AlpacaResourceReader::new(io_provider.clone())?);

        let pipeline_layout_registry = Arc::new(PipelineLayoutRegistry::create(
            &resource_factories.pipeline_layout_factory,
            &descriptor_set_manager,
        )?);

        let scene_loader = Arc::new(SceneLoader::create(alpaca_resource_reader.clone()));

        let skeletons_provider = ResourceProvider::from(
            SkeletonBackend::new(
                resource_context.buffer_manager.clone(),
                persistent_resources.clone(),
                descriptor_index_managers.clone(),
                alpaca_resource_reader.clone(),
                resource_context.resource_loader.clone(),
            ),
            descriptor_index_managers.skeletons_index_manager.clone(),
            frame_counter.clone(),
        );

        let image_provider = ResourceProvider::from(
            ImageBackend::new(
                resource_factories.clone(),
                alpaca_resource_reader.clone(),
                persistent_resources.clone(),
                descriptor_set_manager.clone(),
                resource_context.resource_loader.clone(),
            ),
            descriptor_index_managers.texture_descriptors_index_manager.clone(),
            frame_counter.clone(),
        );
        
        let material_provider = ResourceProvider::from(
            MaterialBackend::new(
                resource_context.buffer_manager.clone(),
                image_provider.clone(),
                alpaca_resource_reader.clone(),
                resource_context.resource_loader.clone(),
                &persistent_resources,
            ),
            descriptor_index_managers.material_index_manager.clone(),
            frame_counter.clone(),
        );

        let mesh_provider = ResourceProvider::from(
            MeshBackend::new(
                resource_context.buffer_manager.clone(),
                alpaca_resource_reader.clone(),
                descriptor_index_managers.clone(),
                persistent_resources.clone(),
                material_provider.clone(),
                skeletons_provider.clone(),
                resource_context.resource_loader.clone(),
            ),
            descriptor_index_managers.mesh_index_manager.clone(),
            frame_counter.clone(),
        );

        let pipeline_provider = ResourceProvider::from(
            PipelineBackend::new(
                device_context.device.clone(),
                device_context.debug_utils.clone(),
                PipelineCache::null(),
                alpaca_resource_reader.clone(),
                pipeline_layout_registry.clone(),
            ),
            descriptor_index_managers.pipeline_index_manager.clone(),
            frame_counter.clone(),
        );

        let compute_pipeline_provider = ResourceProvider::from(
            ComputePipelineBackend::new(
                device_context.device.clone(),
                device_context.debug_utils.clone(),
                PipelineCache::null(),
                alpaca_resource_reader.clone(),
                pipeline_layout_registry.clone(),
            ),
            descriptor_index_managers.compute_pipeline_index_manager.clone(),
            frame_counter.clone(),
        );

        Ok(Self {
            scene_loader,

            alpaca_resource_reader,

            descriptor_set_manager,
            pipeline_layout_registry,

            image_provider,
            material_provider,
            skeletons_provider,
            mesh_provider,
            pipeline_provider,
            compute_pipeline_provider,
        })
    }

    pub fn get_resource_loader(&self) -> Arc<AlpacaResourceReader> {
        self.alpaca_resource_reader.clone()
    }

    pub fn get_mesh_provider(&self) -> Arc<ResourceProvider<MeshBackend>> {
        self.mesh_provider.clone()
    }

    pub fn get_pipeline_provider(&self) -> Arc<ResourceProvider<PipelineBackend>> {
        self.pipeline_provider.clone()
    }

    pub fn get_compute_pipeline_provider(&self) -> Arc<ResourceProvider<ComputePipelineBackend>> {
        self.compute_pipeline_provider.clone()
    }

    pub fn update(&self) {
        self.image_provider.update();
        self.material_provider.update();
        self.skeletons_provider.update();
        self.mesh_provider.update();
        self.pipeline_provider.update();
        self.compute_pipeline_provider.update();
    }

    pub fn destroy(
        self,
        pipeline_layout_factory: &PipelineLayoutFactory,
        sampler_factory: &SamplerFactory,
        descriptor_set_layout_factory: &DescriptorSetLayoutFactory,
    ) {
        self.pipeline_provider.destroy();
        self.compute_pipeline_provider.destroy();
        self.skeletons_provider.destroy();
        self.mesh_provider.destroy();
        self.material_provider.destroy();
        self.image_provider.destroy();

        self.descriptor_set_manager.destroy(&sampler_factory, &descriptor_set_layout_factory);
        self.pipeline_layout_registry.destroy(&pipeline_layout_factory);
    }
}
