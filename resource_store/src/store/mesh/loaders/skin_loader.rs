use crate::store::mesh::backend::mesh_backend::MeshBackend;
use crate::store::mesh::backend::mesh_config::MeshConfig;
use anyhow::{bail, Context, Result};
use resource_residency::ResRef;
use resource_residency::ResourceProvider;
use std::sync::Arc;

pub struct SkinLoader {
    slice_count: u32,

    mesh_provider: Arc<ResourceProvider<MeshBackend>>,
}

impl SkinLoader {
    pub fn new(slice_count: u32, mesh_provider: Arc<ResourceProvider<MeshBackend>>) -> Result<Self> {
        if slice_count < 2 {
            bail!("Skin slice count must be at least 2, got {}", slice_count);
        }

        Ok(Self {
            slice_count,

            mesh_provider,
        })
    }

    pub fn load(&self, mesh: &Arc<ResRef>) -> Result<Arc<ResRef>> {
        let skinned = self.mesh_provider.backend
            .with_mesh(mesh.id, |original| original.skeletal.is_some())
            .context("Skin mesh is not resident")?;

        if !skinned {
            bail!("Skin mesh has no vertex skins");
        }

        let skin = self.mesh_provider.reserve()?;

        self.mesh_provider.write(&skin, MeshConfig::Instance {
            original: mesh.clone(),

            vertex_slice_count: self.slice_count,
            bounds: None,
        })?;

        Ok(skin)
    }
}
