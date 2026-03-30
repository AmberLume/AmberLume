use crate::resources::resource_factories::ResourceFactories;
use anyhow::Result;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::resources::descriptor_index_managers::IndexManagers;
use crate::resources::descriptor_set_manager::DescriptorSetManager;
use crate::resources::persistent::persistent_images::PersistentImages;
use crate::resources::persistent::persistent_materials::PersistentMaterials;
use crate::resources::persistent::persistent_meshes::PersistentMeshes;
use crate::resources::persistent::persistent_shadows::PersistentShadows;
use crate::resources::persistent::persistent_skeletons::PersistentSkeletons;

pub struct PersistentResources {
    pub images: PersistentImages,
    pub skeletons: PersistentSkeletons,
    pub materials: PersistentMaterials,
    pub meshes: PersistentMeshes,
    pub shadows: PersistentShadows,
}

impl PersistentResources {
    pub fn create(
        skeletons: PersistentSkeletons,
        images: PersistentImages,
        materials: PersistentMaterials,
        meshes: PersistentMeshes,
        descriptor_set_manager: &DescriptorSetManager,
        resource_factories: &ResourceFactories,
        renderer_limits: &RendererLimits,
        index_managers: &IndexManagers,
    ) -> Result<Self> {
        let shadows = PersistentShadows::create(
            &index_managers,
            &resource_factories.managed_image_factory,
            &renderer_limits,
            &descriptor_set_manager,
        )?;

        Ok(Self {
            images,
            skeletons,
            materials,
            meshes,
            shadows,
        })
    }

    pub fn destroy(
        self,
        index_managers: &IndexManagers,
        image_factory: &ManagedImageFactory,
    ) -> Result<()> {
        self.shadows.destroy(&index_managers, &image_factory)?;
        self.meshes.destroy();
        self.materials.destroy();
        self.skeletons.destroy();
        self.images.destroy();

        Ok(())
    }
}
