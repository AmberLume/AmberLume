use gltf::Texture as GltfTexture;
use gltf::image::Source;

#[derive(Debug, PartialEq, Clone)]
pub struct Texture {
    pub uri: String,
}

impl Texture {
    pub fn adapt(texture: &GltfTexture) -> Option<Texture> {
        match texture.source().source() {
            Source::Uri { uri, .. } => Some(Self {
                uri: uri.to_string(),
            }),
            Source::View { .. } => None,
        }
    }
}
