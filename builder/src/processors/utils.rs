use crate::build_target::BuildTarget;
use resource_data::resource_key::ResourceKey;

pub fn resource_key(build_target: &BuildTarget, name: &str, extension: &str) -> ResourceKey {
    let key = build_target.relative.join(name).with_added_extension(extension);

    ResourceKey {
        value: key.to_str().unwrap().to_string(),
    }
}

