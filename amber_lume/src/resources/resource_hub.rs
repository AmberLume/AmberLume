use crate::platform_providers::io_provider::IOProvider;
use crate::render::resources::resource_context::ResourceContext;
use crate::render::device::device_context::DeviceContext;
use crate::resources::alpaca_resource_reader::alpaca_resource_reader::AlpacaResourceReader;
use crate::resources::dynamic::mesh::mesh_backend::MeshBackend;
use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use ash::vk::{PipelineCache, SampleCountFlags};
use crate::limits::renderer_limits::RendererLimits;
use crate::render::factories::descriptor_set_layout::descriptor_set_layout_factory::DescriptorSetLayoutFactory;
use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::render::factories::pipeline_layout::pipeline_layout_factory::PipelineLayoutFactory;
use crate::render::factories::sampler::sampler_factory::SamplerFactory;
use crate::render::swapchain::swapchain_context::SwapchainContext;
use crate::resources::dynamic::image::image_backend::ImageBackend;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::index_managers::IndexManagers;
use crate::resources::descriptor_set_manager::DescriptorSetManager;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::dynamic::material::material_backend::MaterialBackend;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::skeleton::skeleton_backend::SkeletonBackend;
use crate::resources::persistent::persistent_images::PersistentImages;
use crate::resources::persistent::persistent_materials::PersistentMaterials;
use crate::resources::persistent::persistent_meshes::PersistentMeshes;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::resources::persistent::persistent_skeletons::PersistentSkeletons;
use crate::resources::pipeline_layout_registry::PipelineLayoutRegistry;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::resources::resource_hub_statistics::ResourcesStatistics;
use crate::resources::scene_loader::scene_loader::SceneLoader;
use crate::utils::arc_utils::ArcUnwrapOrErr;

pub struct ResourceHub {
    pub scene_loader: Arc<SceneLoader>,

    pub alpaca_resource_reader: Arc<AlpacaResourceReader>,

    pub descriptor_set_manager: Arc<DescriptorSetManager>,
    pub pipeline_layout_registry: Arc<PipelineLayoutRegistry>,

    pub image_provider: Arc<ResourceProvider<ImageBackend>>,
    pub material_provider: Arc<ResourceProvider<MaterialBackend>>,
    pub skeletons_provider: Arc<ResourceProvider<SkeletonBackend>>,
    pub mesh_provider: Arc<ResourceProvider<MeshBackend>>,
    pub pipeline_provider: Arc<ResourceProvider<PipelineBackend>>,
    pub compute_pipeline_provider: Arc<ResourceProvider<ComputePipelineBackend>>,

    pub persistent_resources: Arc<PersistentResources>,
}

impl ResourceHub {
    pub fn create(
        device_context: &DeviceContext,
        resource_context: &mut ResourceContext,
        swapchain_context: &SwapchainContext,
        renderer_limits: &RendererLimits,
        descriptor_set_manager: Arc<DescriptorSetManager>,
        descriptor_index_managers: Arc<IndexManagers>,
        frame_counter: Arc<AtomicU64>,
        resource_factories: Arc<ResourceFactories>,
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
                &renderer_limits,
                resource_context.buffer_manager.clone(),
                alpaca_resource_reader.clone(),
                resource_context.resource_loader.clone(),
                frame_counter.clone(),
            ),
            renderer_limits.render_resource_limits.max_skeletons,
            renderer_limits.frames_in_flight,
            frame_counter.clone(),
        );

        let persistent_skeletons = PersistentSkeletons::create(
            &skeletons_provider,
            &renderer_limits,
        )?;

        let image_provider = ResourceProvider::from(
            ImageBackend::new(
                resource_factories.clone(),
                alpaca_resource_reader.clone(),
                descriptor_set_manager.clone(),
                resource_context.resource_loader.clone(),
            ),
            renderer_limits.image_resource_limits.max_texture_descriptors,
            renderer_limits.frames_in_flight,
            frame_counter.clone(),
        );

        let persistent_images = PersistentImages::create(
            &image_provider,
            swapchain_context.format,
            SampleCountFlags::TYPE_1,
        )?;
        
        let material_provider = ResourceProvider::from(
            MaterialBackend::new(
                resource_context.buffer_manager.clone(),
                image_provider.clone(),
                alpaca_resource_reader.clone(),
                resource_context.resource_loader.clone(),
                &persistent_images,
            ),
            renderer_limits.render_resource_limits.max_materials,
            renderer_limits.frames_in_flight,
            frame_counter.clone(),
        );

        let persistent_materials = PersistentMaterials::create(
            &material_provider,
            &persistent_images,
        )?;

        let mesh_provider = ResourceProvider::from(
            MeshBackend::new(
                resource_context.buffer_manager.clone(),
                alpaca_resource_reader.clone(),
                descriptor_index_managers.clone(),
                material_provider.clone(),
                skeletons_provider.clone(),
                &persistent_materials,
                resource_context.resource_loader.clone(),
            ),
            renderer_limits.render_resource_limits.max_meshes,
            renderer_limits.frames_in_flight,
            frame_counter.clone(),
        );

        let persistent_meshes = PersistentMeshes::create(
            &mesh_provider,
            &persistent_materials,
        )?;

        let pipeline_provider = ResourceProvider::from(
            PipelineBackend::new(
                device_context.device.clone(),
                device_context.debug_utils.clone(),
                PipelineCache::null(),
                alpaca_resource_reader.clone(),
                pipeline_layout_registry.clone(),
            ),
            128,
            renderer_limits.frames_in_flight,
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
            128,
            renderer_limits.frames_in_flight,
            frame_counter.clone(),
        );

        let persistent_resources = Arc::new(PersistentResources::create(
            persistent_skeletons,
            persistent_images,
            persistent_materials,
            persistent_meshes,
            &descriptor_set_manager,
            &resource_factories,
            &renderer_limits,
            &descriptor_index_managers,
        )?);

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

            persistent_resources,
        })
    }

    pub fn update(&self) {
        self.image_provider.update();
        self.material_provider.update();
        self.skeletons_provider.update();
        self.mesh_provider.update();
        self.pipeline_provider.update();
        self.compute_pipeline_provider.update();
    }

    pub fn statistics(&self) -> ResourcesStatistics {
        ResourcesStatistics {
            image_provider: self.image_provider.usage_statistics(),
            skeleton_provider: self.skeletons_provider.usage_statistics(),
            material_provider: self.material_provider.usage_statistics(),
            mesh_provider: self.mesh_provider.usage_statistics(),
            pipeline_provider: self.pipeline_provider.usage_statistics(),
            compute_pipeline_provider: self.compute_pipeline_provider.usage_statistics(),
        }
    }
    
    pub fn destroy(
        self,
        index_managers: &IndexManagers,
        image_factory: &ManagedImageFactory,
        sampler_factory: &SamplerFactory,
        pipeline_layout_factory: &PipelineLayoutFactory,
        descriptor_set_layout_factory: &DescriptorSetLayoutFactory,
    ) -> Result<()> {
        self.persistent_resources.try_unwrap()?.destroy(index_managers, image_factory)?;

        self.pipeline_provider.try_unwrap()?.destroy();
        self.compute_pipeline_provider.try_unwrap()?.destroy();
        self.mesh_provider.try_unwrap()?.destroy();
        self.skeletons_provider.try_unwrap()?.destroy();
        self.material_provider.try_unwrap()?.destroy();
        self.image_provider.try_unwrap()?.destroy();

        self.descriptor_set_manager.try_unwrap()?.destroy(&sampler_factory, &descriptor_set_layout_factory);
        self.pipeline_layout_registry.try_unwrap()?.destroy(&pipeline_layout_factory);

        Ok(())
    }
}
