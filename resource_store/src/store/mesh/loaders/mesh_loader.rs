use crate::store::material::loaders::material_loader::MaterialLoader;
use crate::store::mesh::backend::mesh_backend::MeshBackend;
use crate::store::mesh::backend::mesh_config::MeshConfig;
use crate::store::mesh::backend::submesh_config::SubmeshConfig;
use crate::store::skeleton::loaders::skeleton_loader::SkeletonLoader;
use anyhow::Result;
use gpu_data::MeshBindingGPU;
use resource_data::mesh_data::ArchivedMeshData;
use resource_reader::ResourceReader;
use resource_residency::ResRef;
use resource_residency::ResourceProvider;
use rkyv::access;
use rkyv::rancor::Error;
use std::sync::Arc;

pub struct MeshLoader {
    resource_reader: Arc<dyn ResourceReader>,

    mesh_provider: Arc<ResourceProvider<MeshBackend>>,

    material_loader: Arc<MaterialLoader>,
    skeleton_loader: Arc<SkeletonLoader>,

    default_material: Arc<ResRef>,
}

impl MeshLoader {
    pub fn new(
        resource_reader: Arc<dyn ResourceReader>,
        mesh_provider: Arc<ResourceProvider<MeshBackend>>,
        material_loader: Arc<MaterialLoader>,
        skeleton_loader: Arc<SkeletonLoader>,
        default_material: Arc<ResRef>,
    ) -> Self {
        Self {
            resource_reader,

            mesh_provider,

            material_loader,
            skeleton_loader,

            default_material,
        }
    }

    pub fn load(self: &Arc<Self>, resource_key: &str) -> Result<Arc<ResRef>> {
        let loader = self.clone();
        let key = resource_key.to_string();

        self.mesh_provider.get_or_load(&resource_key, move || loader.read(&key))
    }

    fn read(&self, resource_key: &str) -> Result<MeshConfig> {
        let mesh_bytes = self.resource_reader.get_resource(resource_key)?;
        let archived_mesh_data = access::<ArchivedMeshData, Error>(&mesh_bytes)?;

        let skinned = archived_mesh_data.skeleton.is_some();

        let submeshes = archived_mesh_data.submeshes
            .iter()
            .map(|submesh_data| {
                let material = if let Some(resource_key) = submesh_data.material.as_ref() {
                    self.material_loader.load(&resource_key.value)?
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
            .map(|skeleton| self.skeleton_loader.load(&skeleton.value))
            .transpose()?;

        Ok(MeshConfig::Data {
            submeshes,
            bindings,

            skeleton,
        })
    }
}
