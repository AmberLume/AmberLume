use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use index_allocator::ResourceLimits;
use gpu::DeviceContext;

use gpu::ResourceFactories;
use gpu::ResourceTransfer;
use gpu::BindingLayout;
use crate::store::providers_statistics::ResourcesStatistics;
use crate::store::providers::animation::animation_backend::AnimationBackend;
use crate::store::providers::image::image_backend::ImageBackend;
use crate::store::providers::material::material_backend::MaterialBackend;
use crate::store::providers::mesh::mesh_load_observer::MeshLoadObserver;
use crate::store::providers::mesh::mesh_backend::MeshBackend;
use resource_residency::ResourceProvider;
use crate::store::providers::skeleton::skeleton_backend::SkeletonBackend;
use crate::store::persistent::persistent_images::PersistentImages;
use crate::store::persistent::persistent_materials::PersistentMaterials;
use crate::store::persistent::persistent_meshes::PersistentMeshes;
use crate::store::persistent::persistent_resources::PersistentResources;
use crate::store::persistent::persistent_skeletons::PersistentSkeletons;
use index_allocator::ArcUnwrapOrErr;
use resource_reader::ResourceReader;
use crate::store::providers::image::texture_format::TextureFormat;

pub struct ResourceStore {
    pub image_provider: Arc<ResourceProvider<ImageBackend>>,
    pub(crate) material_provider: Arc<ResourceProvider<MaterialBackend>>,
    pub skeletons_provider: Arc<ResourceProvider<SkeletonBackend>>,
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
        mesh_load_observer: Arc<dyn MeshLoadObserver>,
        destroy_delay: u32,
        frame_counter: Arc<AtomicU64>,
    ) -> Result<Self> {
        let skeletons_provider = ResourceProvider::from(
            SkeletonBackend::new(
                &limits,
                resource_factories.clone(),
                resource_reader.clone(),
                resource_transfer.clone(),
            )?,
            limits.max_skeletons,
            destroy_delay,
            frame_counter.clone(),
        );

        let persistent_skeletons = PersistentSkeletons::create(
            &skeletons_provider,
            &limits,
        )?;

        let animation_provider = ResourceProvider::from(
            AnimationBackend::new(
                &limits,
                resource_factories.clone(),
                resource_reader.clone(),
                resource_transfer.clone(),
            )?,
            limits.max_animations,
            destroy_delay,
            frame_counter.clone(),
        );

        let image_provider = ResourceProvider::from(
            ImageBackend::new(
                TextureFormat::pick_for_device(&device_context.physical_device_info.features),
                resource_factories.clone(),
                resource_reader.clone(),
                binding_layout.descriptor_set_manager.textures_descriptor_set.clone(),
                resource_transfer.clone(),
            ),
            limits.max_texture_descriptors,
            destroy_delay,
            frame_counter.clone(),
        );

        let persistent_images = PersistentImages::create(
            &image_provider,
            &binding_layout.descriptor_set_manager.textures_descriptor_set,
            limits.max_texture_descriptors,
        )?;

        let material_provider = ResourceProvider::from(
            MaterialBackend::new(
                &limits,
                resource_factories.clone(),
                image_provider.clone(),
                resource_reader.clone(),
                resource_transfer.clone(),
                &persistent_images,
            )?,
            limits.max_materials,
            destroy_delay,
            frame_counter.clone(),
        );

        let persistent_materials = PersistentMaterials::create(
            &material_provider,
            &persistent_images,
        )?;

        let mesh_provider = ResourceProvider::from(
            MeshBackend::new(
                &limits,
                resource_factories.clone(),
                &persistent_materials,
                resource_reader.clone(),
                resource_transfer.clone(),
                material_provider.clone(),
                skeletons_provider.clone(),
                device_context.physical_device_info.supports_ray_tracing(),
            )?,
            limits.max_meshes,
            destroy_delay,
            frame_counter.clone(),
        );

        if device_context.physical_device_info.supports_ray_tracing() {
            mesh_provider.backend.subscribe(mesh_load_observer);
        }

        let persistent_meshes = PersistentMeshes::create(
            &mesh_provider,
            &persistent_materials,
        )?;

        let persistent_resources = Arc::new(PersistentResources::create(
            persistent_skeletons,
            persistent_images,
            persistent_materials,
            persistent_meshes,
        )?);

        Ok(Self {
            image_provider,
            material_provider,
            skeletons_provider,
            animation_provider,
            mesh_provider,

            persistent_resources,
        })
    }

    pub fn update(&self) {
        self.image_provider.update();
        self.material_provider.update();
        self.skeletons_provider.update();
        self.animation_provider.update();
        self.mesh_provider.update();
    }

    pub fn statistics(&self) -> ResourcesStatistics {
        ResourcesStatistics {
            image_provider: self.image_provider.statistics(),
            skeleton_provider: self.skeletons_provider.statistics(),
            animation_provider: self.animation_provider.statistics(),
            material_provider: self.material_provider.statistics(),
            mesh_provider: self.mesh_provider.statistics(),
        }
    }

    pub fn destroy(self) -> Result<()> {
        self.mesh_provider.try_unwrap()?.destroy()?;
        self.animation_provider.try_unwrap()?.destroy()?;
        self.skeletons_provider.try_unwrap()?.destroy()?;
        self.material_provider.try_unwrap()?.destroy()?;
        self.image_provider.try_unwrap()?.destroy()?;

        Ok(())
    }
}
