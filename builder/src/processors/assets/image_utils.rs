use crate::build_task::{BuildTask, ConvertKTX2Task, TextureType};
use crate::dispatcher::Dispatcher;
use crate::paths::Paths;
use blake3::hash;
use gltf::{Texture, image};
use std::fs::canonicalize;
use std::sync::Arc;

pub fn extract_image_info(
    dispatcher: Arc<Dispatcher>,
    paths: &Paths,
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
            let texture_hash = hash(&canonicalized.to_string_lossy().as_bytes()).to_string();

            dispatcher
                .clone()
                .dispatch(BuildTask::ConvertKTX2(ConvertKTX2Task {
                    name: texture_hash.clone(),

                    source_path: canonicalized.clone(),

                    target_path: paths.root .clone(),

                    texture_type,
                }));

            Some(texture_hash)
        }
    }
}
