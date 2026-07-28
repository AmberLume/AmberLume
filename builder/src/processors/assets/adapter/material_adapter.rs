use blake3::Hasher;
use bytemuck::bytes_of;
use gltf::material::AlphaMode as GltfAlphaMode;
use gltf::Material as GltfMaterial;
use gltf::Texture as GltfTexture;
use resource_data::alpha_mode::AlphaMode;
use crate::processors::assets::adapter::texture_adapter::Texture;

#[derive(Debug, PartialEq, Clone)]
pub struct Material {
    pub name: String,

    pub base_color_factor: [f32; 4],
    pub roughness_factor: f32,
    pub metallic_factor: f32,

    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,

    pub base_texture: Option<Texture>,
    pub normal_texture: Option<Texture>,
    pub occlusion_roughness_metallic_texture: Option<Texture>,
}

impl Material {
    pub fn adapt(material: &GltfMaterial) -> Material {
        let pbr_metallic_roughness = material.pbr_metallic_roughness();

        Self {
            name: hash_material(material),

            base_color_factor: pbr_metallic_roughness.base_color_factor(),
            roughness_factor: pbr_metallic_roughness.roughness_factor(),
            metallic_factor: pbr_metallic_roughness.metallic_factor(),

            alpha_mode: match material.alpha_mode() {
                GltfAlphaMode::Opaque => AlphaMode::Opaque,
                GltfAlphaMode::Mask => AlphaMode::Mask,
                GltfAlphaMode::Blend => AlphaMode::Blend,
            },
            alpha_cutoff: material.alpha_cutoff().unwrap_or(AlphaMode::DEFAULT_CUTOFF),

            base_texture: pbr_metallic_roughness
                .base_color_texture()
                .and_then(|info| Texture::adapt(&info.texture())),

            normal_texture: material
                .normal_texture()
                .and_then(|info| Texture::adapt(&info.texture())),

            occlusion_roughness_metallic_texture: pbr_metallic_roughness
                .metallic_roughness_texture()
                .and_then(|info| Texture::adapt(&info.texture())),
        }
    }
}

fn hash_material(material: &GltfMaterial) -> String {
    let mut hasher = Hasher::new();

    let pbr = material.pbr_metallic_roughness();

    hasher.update(bytes_of(&pbr.base_color_factor()));
    if let Some(texture) = pbr.base_color_texture() {
        hash_texture(&mut hasher, &texture.texture())
    } else {
        offset(&mut hasher);
    }

    hasher.update(bytes_of(&material.alpha_cutoff().unwrap_or(AlphaMode::DEFAULT_CUTOFF)));
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

fn hash_texture(hasher: &mut Hasher, texture: &GltfTexture) {
    match Texture::adapt(texture) {
        Some(texture) => hasher.update(texture.uri.as_bytes()),
        None => return offset(hasher),
    };
}

fn offset(hasher: &mut Hasher) {
    hasher.update(&[0]);
}
