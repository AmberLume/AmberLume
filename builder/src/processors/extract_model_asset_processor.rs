use std::fs::canonicalize;
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::{bail, Result};
use blake3::hash;
use gltf::{buffer, image, Material, Primitive, Texture};
use rkyv::rancor::Error;
use rkyv::to_bytes;
use tracing::error;
use crate::aabb_utils::{calculate_aabb, calculate_global_aabb};
use crate::build_task::{BuildTask, ConvertKTX2Task, ExtractModelAssetTask, TextureType, WriteFileTask};
use crate::data::material_data::MaterialData;
use crate::data::mesh_data::MeshData;
use crate::data::model_data::ModelData;
use crate::data::submesh_data::SubmeshData;
use crate::dispatcher::Dispatcher;
use crate::gltf_file::GltfFile;
use crate::paths::Paths;
use crate::processors::processor::Processor;

pub struct ExtractModelAssetProcessor;

impl ExtractModelAssetProcessor {
    pub fn create() -> Self {
        Self {
            
        }
    }

    fn collect_submesh_data(
        &self,
        dispatcher: Arc<Dispatcher>,
        paths: &Paths,
        file_name: &String,
        bin: Option<&[u8]>,
        primitive: &Primitive,
    ) -> Result<SubmeshData> {
        let reader = primitive.reader(|buffer| {
            match buffer.source() {
                buffer::Source::Bin => None,
                buffer::Source::Uri(_) => { bin }
            }
        });

        let indices = if let Some(indices) = reader.read_indices() {
            indices.into_u32().collect::<Vec<u32>>()
        } else {
            bail!("Accessor for indices coordinates not found");
        };

        let positions: Vec<[f32; 3]> = if let Some(positions) = reader.read_positions() {
            positions.collect::<Vec<[f32; 3]>>()
        } else {
            bail!("Accessor for positions not found");
        };

        let uvs: Vec<[f32; 2]> = if let Some(texture_coordinates) = reader.read_tex_coords(0) {
            texture_coordinates.into_f32().collect()
        } else {
            bail!("Accessor for texture coordinates not found");
        };

        let normals: Vec<[f32; 3]> = if let Some(iter) = reader.read_normals() {
            iter.collect::<Vec<[f32; 3]>>()
        } else {
            bail!("Accessor for normal not found");
        };

        let tangents: Vec<[f32; 4]> = if let Some(iter) = reader.read_tangents() {
            iter.collect::<Vec<[f32; 4]>>()
        } else {
            bail!("Accessor for tangent not found");
        };

        let positions_count = positions.len();
        let uvs_count = uvs.len();
        let normals_count = positions.len();
        let tangents_count = tangents.len();

        assert!(
            positions_count == uvs_count && uvs_count == normals_count && normals_count == tangents_count,
            "Model arrays are not equals! Positions: {}, normals: {}, tangents: {}, UVs: {}",
            positions_count, normals_count, tangents_count, uvs_count,
        );

        let bounds = calculate_aabb(positions.iter().copied());

        let material_id = self.collect_material_data(dispatcher, &paths, &file_name, primitive.material());

        Ok(SubmeshData {
            material_id,

            indices,
            positions,
            normals,
            tangents,
            uvs,

            bounds,
        })
    }

    fn collect_material_data(
        &self,
        dispatcher: Arc<Dispatcher>,
        paths: &Paths,
        file_name: &String,
        material: Material,
    ) -> Option<String> {
        let material_index = material.index();

        let material_name = if let Some(material_index) = material_index {
            let material_path = paths.relative
                .join(material_index.to_string());
            let hash = hash(material_path.to_string_lossy().to_string().as_bytes()).to_string();
            PathBuf::from(file_name).join(hash).to_string_lossy().to_string()
        } else {
            return None
        };

        let pbr_metallic_roughness = material.pbr_metallic_roughness();
        
        let base_color_factor = pbr_metallic_roughness.base_color_factor();
        let roughness_factor = pbr_metallic_roughness.roughness_factor();
        let metallic_factor = pbr_metallic_roughness.metallic_factor();

        let base_texture_id = pbr_metallic_roughness
            .base_color_texture()
            .and_then(|base_color_texture_info| {
                self.extract_image_info(dispatcher.clone(), &paths, base_color_texture_info.texture(), TextureType::Color)
            });

        let normal_texture_id = material
            .normal_texture()
            .and_then(|normal_texture_info| {
                self.extract_image_info(dispatcher.clone(), &paths, normal_texture_info.texture(), TextureType::Normal)
            });

        let occlusion_roughness_metalic_texture_id = pbr_metallic_roughness
            .metallic_roughness_texture()
            .and_then(|pbr_texture_info| {
                self.extract_image_info(dispatcher.clone(), &paths, pbr_texture_info.texture(), TextureType::OcclusionRoughnessMetalic)
            });

        let material_data = MaterialData {
            base_color_factor,
            roughness_factor,
            metallic_factor,
            
            base_texture_id,
            normal_texture_id,
            occlusion_roughness_metalic_texture_id,
        };

        let material_bytes = to_bytes::<Error>(&material_data).unwrap().into_vec();

        let path = paths.target
            .join(&material_name)
            .with_extension("material");

        dispatcher.dispatch(BuildTask::WriteFile(WriteFileTask {
            target_path: path,

            data: material_bytes,
        }));

        Some(material_name)
    }

    fn extract_image_info(&self, dispatcher: Arc<Dispatcher>, paths: &Paths, texture: Texture, texture_type: TextureType) -> Option<String> {
        let image = texture.source();

        match image.source() {
            image::Source::View { .. } => {
                error!("Models must not contain view images! Path: {}", paths.relative.display());

                None
            },
            image::Source::Uri { uri, .. } => {
                let image_path = paths.source_file().parent().unwrap().join(uri);
                let canonicalized = canonicalize(image_path).unwrap();
                let texture_hash = hash(&canonicalized.to_string_lossy().as_bytes()).to_string();

                dispatcher.clone().dispatch(BuildTask::ConvertKTX2(ConvertKTX2Task {
                    name: texture_hash.clone(),

                    source_path: canonicalized.clone(),

                    target_path: paths.target.clone(),

                    texture_type,
                }));

                Some(texture_hash)
            },
        }
    }
}

impl Processor<ExtractModelAssetTask> for ExtractModelAssetProcessor {
    fn process(&self, dispatcher: Arc<Dispatcher>, task: &ExtractModelAssetTask) -> Result<()> {
        let path = &task.paths.source.join(&task.file_name).join(&task.collection_name).with_extension("gltf");

        let gltf_file = Arc::new(GltfFile::create(&path)?);

        let document = gltf_file.get_document()?;
        let blob = gltf_file.bin();

        let default_scene = if let Some(default_scene) = document.default_scene() {
            default_scene
        } else {
            bail!("GLTF linked assets must have default scene");
        };

        if default_scene.nodes().count() != 1 {
            bail!("GLTF linked assets must be single node");
        }

        let root_node = default_scene.nodes().next().unwrap();

        let meshes = root_node.children().map(|node| {
            let mesh = node.mesh().unwrap();

            let submeshes = mesh.primitives().map(|primitive| {
                self.collect_submesh_data(dispatcher.clone(), &task.paths, &task.file_name, blob, &primitive).unwrap()
            }).collect::<Vec<_>>();

            let bounds = calculate_aabb(submeshes.iter().flat_map(|p| p.positions.iter().copied()));

            MeshData {
                name: mesh.name().unwrap().into(),

                bounds,

                submeshes,
            }
        }).collect::<Vec<_>>();

        let bounds = calculate_global_aabb(meshes.iter().map(|m| m.bounds));

        let model_data = ModelData {
            bounds,

            meshes,
        };

        let model_bytes = to_bytes::<Error>(&model_data)?.into_vec();

        let path = task.paths.target
            .join(&task.file_name)
            .join(&task.collection_name)
            .with_extension("model");

        dispatcher.dispatch(BuildTask::WriteFile(WriteFileTask {
            target_path: path,

            data: model_bytes,
        }));

        Ok(())
    }
}
