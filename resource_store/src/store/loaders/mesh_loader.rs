use crate::store::providers::material::material_backend::MaterialBackend;
use crate::store::providers::material::material_config::MaterialConfig;
use crate::store::providers::mesh::mesh_backend::MeshBackend;
use crate::store::providers::mesh::mesh_config::MeshConfig;
use crate::store::providers::mesh::submesh_config::SubmeshConfig;
use crate::store::providers::skeleton::skeleton_backend::SkeletonBackend;
use crate::store::providers::skeleton::skeleton_config::SkeletonConfig;
use anyhow::Result;
use gpu_data::MeshBindingGPU;
use parking_lot::Mutex;
use resource_data::mesh_data::ArchivedMeshData;
use resource_reader::ResourceReader;
use resource_residency::ResRef;
use resource_residency::ResourceProvider;
use resource_residency::TaskScheduler;
use rkyv::access;
use rkyv::rancor::Error;
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use tracing::error;

pub struct MeshLoader {
    resource_reader: Arc<dyn ResourceReader>,
    scheduler: Arc<dyn TaskScheduler>,

    mesh_provider: Arc<ResourceProvider<MeshBackend>>,
    material_provider: Arc<ResourceProvider<MaterialBackend>>,
    skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,

    default_material: Arc<ResRef>,

    meshes: Mutex<HashMap<String, Weak<ResRef>>>,
}

impl MeshLoader {
    pub fn new(
        resource_reader: Arc<dyn ResourceReader>,
        scheduler: Arc<dyn TaskScheduler>,
        mesh_provider: Arc<ResourceProvider<MeshBackend>>,
        material_provider: Arc<ResourceProvider<MaterialBackend>>,
        skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
        default_material: Arc<ResRef>,
    ) -> Self {
        Self {
            resource_reader,
            scheduler,

            mesh_provider,
            material_provider,
            skeleton_provider,

            default_material,

            meshes: Mutex::new(HashMap::new()),
        }
    }

    pub fn load(self: &Arc<Self>, resource_key: &str) -> Result<Arc<ResRef>> {
        let mut meshes = self.meshes.lock();

        if let Some(mesh) = meshes.get(resource_key).and_then(|mesh| mesh.upgrade()) {
            return Ok(mesh);
        }

        let mesh = self.mesh_provider.reserve()?;

        meshes.insert(resource_key.to_string(), Arc::downgrade(&mesh));

        let loader = self.clone();
        let resource_key = resource_key.to_string();
        let reserved = mesh.clone();

        self.scheduler.schedule(Box::new(move || {
            if let Err(error) = loader.write(&reserved, &resource_key) {
                error!("Failed to load mesh '{}': {:#}", resource_key, error);
            }
        }));

        Ok(mesh)
    }

    fn write(&self, mesh: &ResRef, resource_key: &str) -> Result<()> {
        let mesh_bytes = self.resource_reader.get_resource(resource_key)?;
        let archived_mesh_data = access::<ArchivedMeshData, Error>(&mesh_bytes)?;

        let skinned = archived_mesh_data.skeleton.is_some();

        let submeshes = archived_mesh_data.submeshes
            .iter()
            .map(|submesh_data| {
                let material = if let Some(resource_key) = submesh_data.material.as_ref() {
                    self.material_provider.get_or_load(MaterialConfig::Alpaca {
                        resource_key: resource_key.value.to_string(),
                    })?
                } else {
                    self.default_material.clone()
                };

                Ok(SubmeshConfig::from_archived(submesh_data, skinned, material))
            })
            .collect::<Result<Vec<_>>>()?;

        let bindings = archived_mesh_data.bones
            .iter()
            .map(|bone| MeshBindingGPU::create(
                bone.inverse_bind_matrix.map(|column| column.map(|value| value.into())),
                bone.bounds.map(|value| value.into()),
            ))
            .collect::<Vec<_>>();

        let skeleton = archived_mesh_data.skeleton
            .as_ref()
            .map(|skeleton| {
                self.skeleton_provider.get_or_load(SkeletonConfig::Alpaca {
                    resource_key: skeleton.value.to_string(),
                })
            })
            .transpose()?;

        self.mesh_provider.write(mesh, MeshConfig::Data {
            submeshes,
            bindings,

            skeleton,
        })
    }
}
