use anyhow::Result;
use std::sync::Arc;
use index_allocator::ResourceLimits;
use gpu::DeviceContext;
use gpu::ResourceFactories;
use gpu::ResourceTransfer;
use gpu::BindingLayout;
use gpu::BindlessBinding;
use crate::store::animation::loaders::animation_loader::AnimationLoader;
use crate::store::image::loaders::image_loader::ImageLoader;
use crate::store::material::loaders::material_loader::MaterialLoader;
use crate::store::mesh::loaders::mesh_loader::MeshLoader;
use crate::store::mesh::loaders::skin_loader::SkinLoader;
use crate::store::resource_buffers::ResourceBuffers;
use crate::store::resources_statistics::ResourcesStatistics;
use crate::store::animation::backend::animation_backend::AnimationBackend;
use crate::store::image::backend::image_backend::ImageBackend;
use crate::store::material::backend::material_backend::MaterialBackend;
use crate::store::mesh::backend::mesh_backend::MeshBackend;
use resource_residency::ResourceProvider;
use crate::store::skeleton::backend::skeleton_backend::SkeletonBackend;
use crate::store::skeleton::loaders::skeleton_loader::SkeletonLoader;
use crate::store::persistent::persistent_images::PersistentImages;
use crate::store::persistent::persistent_materials::PersistentMaterials;
use crate::store::persistent::persistent_resources::PersistentResources;
use index_allocator::ArcUnwrapOrErr;
use index_allocator::DeferredDestroy;
use index_allocator::IndexManager;
use index_allocator::ResourceId;
use resource_reader::ResourceReader;
use crate::store::image::loaders::texture_format::TextureFormat;

pub struct ResourceStore {
    resource_factories: Arc<ResourceFactories>,

    pub buffers: ResourceBuffers,
    pub textures: Arc<BindlessBinding>,

    pub image_provider: Arc<ResourceProvider<ImageBackend>>,
    pub(crate) material_provider: Arc<ResourceProvider<MaterialBackend>>,
    pub skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
    pub animation_provider: Arc<ResourceProvider<AnimationBackend>>,
    pub mesh_provider: Arc<ResourceProvider<MeshBackend>>,

    pub image_loader: Arc<ImageLoader>,
    pub material_loader: Arc<MaterialLoader>,
    pub skeleton_loader: Arc<SkeletonLoader>,
    pub animation_loader: Arc<AnimationLoader>,
    pub mesh_loader: Arc<MeshLoader>,
    pub skin_loader: Arc<SkinLoader>,

    pub persistent_resources: Arc<PersistentResources>,
}

impl ResourceStore {
    pub fn new(
        limits: &ResourceLimits,
        device_context: &DeviceContext,
        binding_layout: Arc<BindingLayout>,
        resource_reader: Arc<dyn ResourceReader>,
        resource_transfer: Arc<ResourceTransfer>,
        resource_factories: Arc<ResourceFactories>,
        deferred_destroy: Arc<DeferredDestroy>,
    ) -> Result<Self> {
        let buffers = ResourceBuffers::create(
            &resource_factories.buffer_factory,
            limits,
            device_context.physical_device_info.supports_ray_tracing(),
        )?;

        let textures = Arc::new(BindlessBinding::new(
            binding_layout.descriptor_set_manager.textures_descriptor_set.clone(),
            IndexManager::new(limits.max_texture_descriptors),
            deferred_destroy.clone(),
        ));

        let skeleton_provider = ResourceProvider::from(
            SkeletonBackend::new(
                resource_transfer.clone(),
                buffers.skeleton.clone(),
                buffers.skeleton_bone.clone(),
            ),
            buffers.skeleton.allocator.clone(),
            deferred_destroy.clone(),
        );

        let animation_provider = ResourceProvider::from(
            AnimationBackend::new(
                resource_transfer.clone(),
                buffers.animation.clone(),
                buffers.animation_frame.clone(),
            ),
            buffers.animation.allocator.clone(),
            deferred_destroy.clone(),
        );

        let image_provider = ResourceProvider::from(
            ImageBackend::new(
                resource_factories.clone(),
                resource_transfer.clone(),
                textures.clone(),
            )?,
            textures.index_manager.clone(),
            deferred_destroy.clone(),
        );

        let persistent_images = PersistentImages::create(&image_provider)?;

        let material_provider = ResourceProvider::from(
            MaterialBackend::new(
                resource_transfer.clone(),
                buffers.material.clone(),
            ),
            buffers.material.allocator.clone(),
            deferred_destroy.clone(),
        );

        let persistent_materials = PersistentMaterials::create(
            &material_provider,
            &persistent_images,
        )?;

        let mesh_provider = ResourceProvider::from(
            MeshBackend::new(
                resource_transfer.clone(),
                buffers.mesh.clone(),
                buffers.submesh.clone(),
                buffers.index.clone(),
                buffers.vertex.clone(),
                buffers.vertex_uv.clone(),
                buffers.submesh_bounds.clone(),
                buffers.mesh_vertex_skin.clone(),
                buffers.mesh_binding.clone(),
            ),
            buffers.mesh.allocator.clone(),
            deferred_destroy.clone(),
        );

        let image_loader = Arc::new(ImageLoader::new(
            resource_reader.clone(),
            TextureFormat::pick_for_device(&device_context.physical_device_info.features),
            image_provider.clone(),
        ));

        let material_loader = Arc::new(MaterialLoader::new(
            resource_reader.clone(),
            material_provider.clone(),
            image_loader.clone(),
            &persistent_images,
        ));

        let skeleton_loader = Arc::new(SkeletonLoader::new(
            resource_reader.clone(),
            skeleton_provider.clone(),
        ));

        let animation_loader = Arc::new(AnimationLoader::new(
            resource_reader.clone(),
            animation_provider.clone(),
            skeleton_loader.clone(),
        ));

        let mesh_loader = Arc::new(MeshLoader::new(
            resource_reader.clone(),
            mesh_provider.clone(),
            material_loader.clone(),
            skeleton_loader.clone(),
            persistent_materials.default.clone(),
        ));

        let skin_loader = Arc::new(SkinLoader::new(
            limits.skin_slice_count,
            mesh_provider.clone(),
        )?);

        let persistent_resources = Arc::new(PersistentResources::create(
            persistent_images,
            persistent_materials,
        ));

        Ok(Self {
            resource_factories,

            buffers,
            textures,

            image_provider,
            material_provider,
            skeleton_provider,
            animation_provider,
            mesh_provider,

            image_loader,
            material_loader,
            skeleton_loader,
            animation_loader,
            mesh_loader,
            skin_loader,

            persistent_resources,
        })
    }

    pub fn update(&self) -> Vec<ResourceId> {
        self.image_provider.update();
        self.material_provider.update();
        self.skeleton_provider.update();
        self.animation_provider.update();
        self.mesh_provider.update()
    }

    pub fn statistics(&self) -> ResourcesStatistics {
        ResourcesStatistics {
            image_provider: self.image_provider.statistics(),
            skeleton_provider: self.skeleton_provider.statistics(),
            animation_provider: self.animation_provider.statistics(),
            material_provider: self.material_provider.statistics(),
            mesh_provider: self.mesh_provider.statistics(),
        }
    }

    pub fn destroy(self) -> Result<()> {
        self.skin_loader.try_unwrap()?;
        self.mesh_loader.try_unwrap()?;
        self.animation_loader.try_unwrap()?;
        self.skeleton_loader.try_unwrap()?;
        self.material_loader.try_unwrap()?;
        self.image_loader.try_unwrap()?;

        self.mesh_provider.try_unwrap()?.destroy()?;
        self.animation_provider.try_unwrap()?.destroy()?;
        self.skeleton_provider.try_unwrap()?.destroy()?;
        self.material_provider.try_unwrap()?.destroy()?;
        self.image_provider.try_unwrap()?.destroy()?;

        self.buffers.destroy(&self.resource_factories.buffer_factory)?;

        Ok(())
    }
}
