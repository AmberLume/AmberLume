use std::collections::HashMap;
use std::fs::write;
use std::path::{Path, PathBuf};
use anyhow::Result;

pub struct Category {
    pub module: &'static str,
    pub handle: &'static str,
    pub files: Vec<PathBuf>,
}

pub fn generate_manifest(generated_dir: &Path, categories: &[Category]) -> Result<()> {
    let mut source = String::new();

    source.push_str("use crate::data::resource_handle::{");
    let mut handles: Vec<&str> = categories.iter().map(|c| c.handle).collect();
    handles.sort_unstable();
    handles.dedup();
    source.push_str(&handles.join(", "));
    source.push_str("};\n\n");

    for category in categories {
        let mut entries: Vec<(String, String)> = category.files
            .iter()
            .map(|path| {
                let key = path.to_str().unwrap().replace('\\', "/");
                (constant_name(path), key)
            })
            .collect();

        entries.sort();
        disambiguate(&mut entries);

        source.push_str(&format!("pub mod {} {{\n", category.module));
        source.push_str(&format!("    use super::{};\n\n", category.handle));

        for (name, key) in &entries {
            source.push_str(&format!(
                "    pub const {}: {} = {}(\"{}\");\n",
                name, category.handle, category.handle, key,
            ));
        }

        source.push_str("}\n\n");
    }

    write(generated_dir.join("resources.rs"), source)?;

    Ok(())
}

fn constant_name(path: &Path) -> String {
    let stem = path.file_stem().unwrap().to_str().unwrap();

    sanitize(stem)
}

fn sanitize(value: &str) -> String {
    let mut result = String::with_capacity(value.len());

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            result.push(character.to_ascii_uppercase());
        } else {
            result.push('_');
        }
    }

    if result.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(true) {
        result.insert(0, '_');
    }

    result
}

fn disambiguate(entries: &mut [(String, String)]) {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (name, _) in entries.iter() {
        *counts.entry(name.clone()).or_insert(0) += 1;
    }

    let mut seen: HashMap<String, usize> = HashMap::new();
    for (name, _) in entries.iter_mut() {
        if counts[name] > 1 {
            let index = seen.entry(name.clone()).or_insert(0);
            let suffixed = format!("{}_{}", name, index);
            *index += 1;
            *name = suffixed;
        }
    }
}
