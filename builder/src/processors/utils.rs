use crate::build_target::BuildTarget;
use resource_data::resource_key::ResourceKey;

pub fn resource_key(build_target: &BuildTarget, name: &str, extension: &str) -> ResourceKey {
    let key = build_target.relative.join(name).with_added_extension(extension);

    ResourceKey {
        value: key.to_str().unwrap().to_string(),
    }
}

pub fn resource_key_from_build_target(build_target: &BuildTarget, extension: &str) -> ResourceKey {
    let key = build_target.relative
        .join(&build_target.name)
        .with_added_extension(&build_target.extension)
        .with_added_extension(extension);

    ResourceKey {
        value: key.to_str().unwrap().to_string(),
    }
}

