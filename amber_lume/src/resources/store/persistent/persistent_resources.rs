use crate::resources::store::persistent::persistent_images::PersistentImages;
use crate::resources::store::persistent::persistent_materials::PersistentMaterials;
use crate::resources::store::persistent::persistent_meshes::PersistentMeshes;
use crate::resources::store::persistent::persistent_skeletons::PersistentSkeletons;
use anyhow::Result;

pub struct PersistentResources {
    pub images: PersistentImages,
    pub skeletons: PersistentSkeletons,
    pub materials: PersistentMaterials,
    pub meshes: PersistentMeshes,
}

impl PersistentResources {
    pub fn create(
        skeletons: PersistentSkeletons,
        images: PersistentImages,
        materials: PersistentMaterials,
        meshes: PersistentMeshes,
    ) -> Result<Self> {
        Ok(Self {
            images,
            skeletons,
            materials,
            meshes,
        })
    }

    pub fn destroy(self) -> Result<()> {
        self.meshes.destroy();
        self.materials.destroy();
        self.skeletons.destroy();
        self.images.destroy();

        Ok(())
    }
}
