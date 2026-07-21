use crate::build_task::{BuildTask, TextureType};
use crate::dispatcher::Dispatcher;
use rkyv::rancor::Error;
use rkyv::to_bytes;
use std::sync::Arc;
use crate::build_target::BuildTarget;
use resource_data::material_data::MaterialData;
use resource_data::resource_key::ResourceKey;
use crate::processors::assets::adapter::material_adapter::Material;
use crate::processors::assets::writer::image_writer::write_image;
use crate::processors::utils::resource_key;

pub fn write_material_data(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    material: &Material,
) -> Option<ResourceKey> {
    let base_texture_id = material.base_texture
        .as_ref()
        .and_then(|texture| write_image(dispatcher.clone(), build_target, texture, TextureType::Color));

    let normal_texture_id = material.normal_texture
        .as_ref()
        .and_then(|texture| write_image(dispatcher.clone(), build_target, texture, TextureType::Normal));

    let occlusion_roughness_metallic_texture_id = material.occlusion_roughness_metallic_texture
        .as_ref()
        .and_then(|texture| write_image(dispatcher.clone(), build_target, texture, TextureType::OcclusionRoughnessMetallic));

    let resource_key = resource_key(build_target, &material.name, "MATERIAL");
    dispatcher.dispatch(BuildTask::archive(
        build_target,
        &resource_key,
        to_bytes::<Error>(&MaterialData {
            base_color_factor: material.base_color_factor,
            roughness_factor: material.roughness_factor,
            metallic_factor: material.metallic_factor,

            base_texture_id,
            normal_texture_id,
            occlusion_roughness_metallic_texture_id,
        }).ok()?.to_vec(),
    ));

    Some(resource_key)
}
