use crate::build_task::{BuildTask, ConvertKTX2Task, TextureType};
use crate::dispatcher::Dispatcher;
use blake3::hash;
use gltf::{Texture, image};
use std::fs::{canonicalize, read};
use std::sync::Arc;
use crate::build_target::BuildTarget;
use resource_data::resource_key::ResourceKey;
use crate::processors::utils::resource_key;

pub fn write_image(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    texture: Texture,
    texture_type: TextureType,
) -> Option<ResourceKey> {
    let image = texture.source();

    match image.source() {
        image::Source::View { .. } => {
            None
        }
        image::Source::Uri { uri, .. } => {
            let image_path = build_target.entry.parent()?.join(uri);
            let canonicalized = canonicalize(image_path).ok()?;
            let texture_bytes = read(&canonicalized).ok()?;
            let texture_hash = hash(&texture_bytes).to_string();

            let resource_key = resource_key(&build_target, &texture_hash, "ktx2");
            dispatcher
                .clone()
                .dispatch(BuildTask::ConvertKTX2(ConvertKTX2Task {
                    build_target: build_target.clone(),
                    
                    resource_key: resource_key.value.clone(),
                    
                    source: canonicalized.clone(),
                    
                    texture_type,
                }));

            Some(resource_key)
        }
    }
}
