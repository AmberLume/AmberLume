use crate::build_task::{BuildTask, TextureType};
use crate::dispatcher::Dispatcher;
use blake3::Hasher;
use gltf::{Material, Texture};
use rkyv::rancor::Error;
use rkyv::to_bytes;
use std::sync::Arc;
use bytemuck::bytes_of;
use gltf::image::Source;
use tracing::error;
use crate::build_target::BuildTarget;
use crate::data::material_data::MaterialData;
use crate::data::resource_key::ResourceKey;
use crate::processors::assets::image_utils::write_image;
use crate::processors::utils::resource_key;

pub fn write_material_data(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    material: Material,
) -> Option<ResourceKey> {
    let name = hash_material(&material);

    let pbr_metallic_roughness = material.pbr_metallic_roughness();

    let base_color_factor = pbr_metallic_roughness.base_color_factor();
    let roughness_factor = pbr_metallic_roughness.roughness_factor();
    let metallic_factor = pbr_metallic_roughness.metallic_factor();

    let base_texture_id = pbr_metallic_roughness.base_color_texture()
        .and_then(|base_color_texture_info| {
            write_image(
                dispatcher.clone(),
                &build_target,
                base_color_texture_info.texture(),
                TextureType::Color,
            )
        });

    let normal_texture_id = material.normal_texture()
        .and_then(|normal_texture_info| {
            write_image(
                dispatcher.clone(),
                &build_target,
                normal_texture_info.texture(),
                TextureType::Normal,
            )
        });

    let occlusion_roughness_metallic_texture_id = pbr_metallic_roughness.metallic_roughness_texture()
        .and_then(|pbr_texture_info| {
            write_image(
                dispatcher.clone(),
                &build_target,
                pbr_texture_info.texture(),
                TextureType::OcclusionRoughnessMetallic,
            )
        });

    let resource_key = resource_key(build_target, &name, "MATERIAL");
    dispatcher.dispatch(BuildTask::archive(
        build_target,
        &resource_key,
        to_bytes::<Error>(&MaterialData {
            base_color_factor,
            roughness_factor,
            metallic_factor,

            base_texture_id,
            normal_texture_id,
            occlusion_roughness_metallic_texture_id,
        }).ok()?.to_vec(),
    ));

    Some(resource_key)
}

fn hash_material(material: &Material) -> String {
    let mut hasher = Hasher::new();

    let pbr = material.pbr_metallic_roughness();

    hasher.update(bytes_of(&pbr.base_color_factor()));
    if let Some(texture) = pbr.base_color_texture() {
        hash_texture(&mut hasher, &texture.texture())
    } else {
        offset(&mut hasher);
    }

    hasher.update(bytes_of(&material.alpha_cutoff().unwrap_or(0.5)));
    hasher.update(&[material.alpha_mode() as u8]);

    hasher.update(&[material.double_sided() as u8]);

    hasher.update(bytes_of(&pbr.metallic_factor()));
    hasher.update(bytes_of(&pbr.roughness_factor()));
    if let Some(texture) = pbr.metallic_roughness_texture() {
        hash_texture(&mut hasher, &texture.texture())
    } else {
        offset(&mut hasher);
    }

    if let Some(texture) = material.normal_texture() {
        hasher.update(bytes_of(&texture.scale()));
        hash_texture(&mut hasher, &texture.texture())
    } else {
        offset(&mut hasher);
    }

    hasher.update(bytes_of(&material.emissive_factor()));
    if let Some(texture) = material.emissive_texture() {
        hash_texture(&mut hasher, &texture.texture())
    } else {
        offset(&mut hasher);
    }

    if let Some(texture) = material.occlusion_texture() {
        hash_texture(&mut hasher, &texture.texture())
    } else {
        offset(&mut hasher);
    }

    hasher.finalize().to_string()
}

fn hash_texture<'a>(hasher: &mut Hasher, texture: &Texture<'a>) {
    match texture.source().source() {
        Source::Uri { uri, .. } => {
            hasher.update(uri.as_bytes());
        },
        Source::View { .. } => {
            error!("Material View source not supported");

            offset(hasher);
        },
    }
}

fn offset(hasher: &mut Hasher) {
    hasher.update(&[0]);
}
