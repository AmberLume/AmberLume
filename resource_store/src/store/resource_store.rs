use anyhow::Result;
use std::sync::Arc;
use index_allocator::ResourceLimits;
use gpu::DeviceContext;
use gpu::ResourceFactories;
use gpu::ResourceTransfer;
use gpu::BindingLayout;
use gpu::BindlessBinding;
use crate::store::mesh_table::mesh_table::MeshTable;
use crate::store::resource_buffers::ResourceBuffers;
use crate::store::resources_statistics::ResourcesStatistics;
use crate::store::providers::animation::animation_backend::AnimationBackend;
use crate::store::providers::image::image_backend::ImageBackend;
use crate::store::providers::material::material_backend::MaterialBackend;
use crate::store::providers::mesh::mesh_backend::MeshBackend;
use resource_residency::ResourceProvider;
use crate::store::providers::skeleton::skeleton_backend::SkeletonBackend;
use crate::store::persistent::persistent_images::PersistentImages;
use crate::store::persistent::persistent_materials::PersistentMaterials;
use crate::store::persistent::persistent_resources::PersistentResources;
use index_allocator::ArcUnwrapOrErr;
use index_allocator::DeferredDestroy;
use index_allocator::IndexManager;
use resource_reader::ResourceReader;
use crate::store::providers::image::texture_format::TextureFormat;

pub struct ResourceStore {
    resource_factories: Arc<ResourceFactories>,

    pub buffers: ResourceBuffers,
    pub textures: Arc<BindlessBinding>,

    pub mesh_table: Arc<MeshTable>,

    pub image_provider: Arc<ResourceProvider<ImageBackend>>,
    pub(crate) material_provider: Arc<ResourceProvider<MaterialBackend>>,
    pub skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
    pub animation_provider: Arc<ResourceProvider<AnimationBackend>>,
    pub mesh_provider: Arc<ResourceProvider<MeshBackend>>,

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

        let mesh_table = Arc::new(MeshTable::create(
            buffers.mesh.clone(),
            buffers.submesh.clone(),
            resource_transfer.clone(),
        ));

        let textures = Arc::new(BindlessBinding::new(
            binding_layout.descriptor_set_manager.textures_descriptor_set.clone(),
            IndexManager::new(limits.max_texture_descriptors),
            deferred_destroy.clone(),
        ));

        let skeleton_provider = ResourceProvider::from(
            SkeletonBackend::new(
                buffers.skeleton.clone(),
                buffers.skeleton_bone.clone(),
                resource_reader.clone(),
                resource_transfer.clone(),
            ),
            buffers.skeleton.allocator.clone(),
            deferred_destroy.clone(),
        );

        let animation_provider = ResourceProvider::from(
            AnimationBackend::new(
                buffers.animation.clone(),
                buffers.animation_frame.clone(),
                resource_reader.clone(),
                resource_transfer.clone(),
                skeleton_provider.clone(),
            ),
            buffers.animation.allocator.clone(),
            deferred_destroy.clone(),
        );

        let image_provider = ResourceProvider::from(
            ImageBackend::new(
                TextureFormat::pick_for_device(&device_context.physical_device_info.features),
                resource_factories.clone(),
                resource_reader.clone(),
                textures.clone(),
                resource_transfer.clone(),
            ),
            textures.index_manager.clone(),
            deferred_destroy.clone(),
        );

        let persistent_images = PersistentImages::create(
            &image_provider,
            &textures.descriptor_set,
            limits.max_texture_descriptors,
        )?;

        let material_provider = ResourceProvider::from(
            MaterialBackend::new(
                &limits,
                buffers.material.clone(),
                image_provider.clone(),
                resource_reader.clone(),
                resource_transfer.clone(),
                &persistent_images,
            )?,
            buffers.material.allocator.clone(),
            deferred_destroy.clone(),
        );

        let persistent_materials = PersistentMaterials::create(
            &material_provider,
            &persistent_images,
        )?;

        let mesh_provider = ResourceProvider::from(
            MeshBackend::new(
                mesh_table.clone(),
                buffers.index.clone(),
                buffers.mesh_vertex.clone(),
                buffers.mesh_vertex_attribute.clone(),
                buffers.mesh_vertex_skin.clone(),
                buffers.mesh_bone.clone(),
                &persistent_materials,
                resource_reader.clone(),
                resource_transfer.clone(),
                material_provider.clone(),
                skeleton_provider.clone(),
            ),
            buffers.mesh.allocator.clone(),
            deferred_destroy.clone(),
        );

        let persistent_resources = Arc::new(PersistentResources::create(
            persistent_images,
            persistent_materials,
        ));

        Ok(Self {
            resource_factories,

            buffers,
            textures,

            mesh_table,

            image_provider,
            material_provider,
            skeleton_provider,
            animation_provider,
            mesh_provider,

            persistent_resources,
        })
    }

    pub fn update(&self) {
        self.image_provider.update();
        self.material_provider.update();
        self.skeleton_provider.update();
        self.animation_provider.update();
        self.mesh_provider.update();
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
        self.mesh_provider.try_unwrap()?.destroy()?;
        self.animation_provider.try_unwrap()?.destroy()?;
        self.skeleton_provider.try_unwrap()?.destroy()?;
        self.material_provider.try_unwrap()?.destroy()?;
        self.image_provider.try_unwrap()?.destroy()?;

        self.mesh_table.try_unwrap()?;

        self.buffers.destroy(&self.resource_factories.buffer_factory)?;

        Ok(())
    }
}
