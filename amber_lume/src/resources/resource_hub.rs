use crate::platform_providers::io_provider::IOProvider;
use crate::render::resources::resource_context::ResourceContext;
use crate::render::device::device_context::DeviceContext;
use crate::resources::index::resource_index::ResourceIndex;
use crate::resources::dynamic::mesh::mesh_backend::MeshBackend;
use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use ash::vk::PipelineCache;
use crate::resources::dynamic::image::image_backend::ImageBackend;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::descriptor_index_managers::IndexManagers;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::dynamic::material::material_backend::MaterialBackend;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::skeleton::skeleton_backend::SkeletonBackend;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::resources::resource_factories::ResourceFactories;
use crate::resources::scene_loader::scene_loader::SceneLoader;

pub struct ResourceHub {
    pub scene_loader: Arc<SceneLoader>,

    resource_loader: Arc<ResourceIndex>,

    image_provider: Arc<ResourceProvider<ImageBackend>>,
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
        descriptor_index_managers: Arc<IndexManagers>,
        frame_counter: Arc<AtomicU64>,
        resource_factories: Arc<ResourceFactories>,
        persistent_resources: Arc<PersistentResources>,
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

        let skeletons_provider = {
            let material_backend = SkeletonBackend::new(
                resource_context.buffer_manager.clone(),
                persistent_resources.clone(),
                descriptor_index_managers.clone(),
                resource_index.clone(),
                resource_context.resource_loader.clone(),
            );

            ResourceProvider::from(
                material_backend,
                descriptor_index_managers.skeletons_index_manager.clone(),
                frame_counter.clone(),
            )
        };

        let image_provider = {
            let image_backend = ImageBackend::new(
                resource_factories.clone(),
                resource_index.clone(),
                persistent_resources.clone(),
                resource_context.resource_loader.clone(),
            );

            ResourceProvider::from(
                image_backend,
                descriptor_index_managers.texture_descriptors_index_manager.clone(),
                frame_counter.clone(),
            )
        };
        
        let material_provider = {
            let material_backend = MaterialBackend::new(
                resource_context.buffer_manager.clone(),
                image_provider.clone(),
                resource_index.clone(),
                resource_context.resource_loader.clone(),
                &persistent_resources,
            );

            ResourceProvider::from(
                material_backend,
                descriptor_index_managers.material_index_manager.clone(),
                frame_counter.clone(),
            )
        };

        let mesh_provider = {
            let mesh_backend = MeshBackend::new(
                resource_context.buffer_manager.clone(),
                resource_index.clone(),
                descriptor_index_managers.clone(),
                persistent_resources.clone(),
                material_provider.clone(),
                skeletons_provider.clone(),
                resource_context.resource_loader.clone(),
            );

            ResourceProvider::from(
                mesh_backend,
                descriptor_index_managers.mesh_index_manager.clone(),
                frame_counter.clone(),
            )
        };

        let pipeline_provider = {
            let pipeline_backend = PipelineBackend::new(
                device_context.device.clone(),
                device_context.debug_utils.clone(),
                PipelineCache::null(),
                resource_index.clone(),
                persistent_resources.clone(),
            );

            ResourceProvider::from(
                pipeline_backend,
                descriptor_index_managers.pipeline_index_manager.clone(),
                frame_counter.clone(),
            )
        };

        let compute_pipeline_provider = {
            let compute_pipeline_backend = ComputePipelineBackend::new(
                device_context.device.clone(),
                device_context.debug_utils.clone(),
                PipelineCache::null(),
                resource_index.clone(),
                persistent_resources.clone(),
            );

            ResourceProvider::from(
                compute_pipeline_backend,
                descriptor_index_managers.compute_pipeline_index_manager.clone(),
                frame_counter.clone(),
            )
        };

        Ok(Self {
            scene_loader,

            resource_loader: resource_index.clone(),

            image_provider,
            material_provider,
            skeletons_provider,
            mesh_provider,
            pipeline_provider,
            compute_pipeline_provider,
        })
    }

    pub fn get_resource_loader(&self) -> Arc<ResourceIndex> {
        self.resource_loader.clone()
    }

    pub fn get_image_provider(&self) -> Arc<ResourceProvider<ImageBackend>> {
        self.image_provider.clone()
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

    pub fn destroy(self) -> Result<()> {
        self.pipeline_provider.destroy();
        self.compute_pipeline_provider.destroy();
        self.skeletons_provider.destroy();
        self.mesh_provider.destroy();
        self.material_provider.destroy();
        self.image_provider.destroy();

        Ok(())
    }
}
