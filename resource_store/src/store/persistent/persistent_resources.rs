use crate::store::persistent::persistent_images::PersistentImages;
use crate::store::persistent::persistent_materials::PersistentMaterials;
use crate::store::persistent::persistent_meshes::PersistentMeshes;
use crate::store::persistent::persistent_skeletons::PersistentSkeletons;
use anyhow::Result;
use resource_residency::ResRef;
use std::sync::Arc;

pub struct PersistentResources {
    images: PersistentImages,
    skeletons: PersistentSkeletons,
    materials: PersistentMaterials,
    meshes: PersistentMeshes,
}

impl PersistentResources {
    pub fn white_pixel(&self) -> Arc<ResRef> {
        self.images.white_pixel.clone()
    }

    pub fn default_material(&self) -> Arc<ResRef> {
        self.materials.default.clone()
    }

    pub(crate) fn create(
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
