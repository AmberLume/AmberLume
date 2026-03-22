use crate::build_task::{BuildTask, ConvertKTX2Task, TextureType};
use crate::dispatcher::Dispatcher;
use crate::paths::AlpacaPaths;
use blake3::hash;
use gltf::{Texture, image};
use std::fs::{canonicalize, read};
use std::sync::Arc;

pub fn extract_image_info(
    dispatcher: Arc<Dispatcher>,
    paths: &AlpacaPaths,
    texture: Texture,
    texture_type: TextureType,
) -> Option<String> {
    let image = texture.source();

    match image.source() {
        image::Source::View { .. } => {
            None
        }
        image::Source::Uri { uri, .. } => {
            let image_path = paths.source_file().parent().unwrap().join(uri);
            let canonicalized = canonicalize(image_path).unwrap();
            let texture_bytes = read(&canonicalized).unwrap();
            let texture_hash = hash(&texture_bytes).to_string();

            dispatcher
                .clone()
                .dispatch(BuildTask::ConvertKTX2(ConvertKTX2Task {
                    name: texture_hash.clone(),

                    source: canonicalized.clone(),
                    target: paths.shared.to_path_buf(),

                    texture_type,
                }));

            Some(texture_hash)
        }
    }
}
