use std::collections::HashMap;
use std::fs::write;
use std::path::{Path, PathBuf};
use anyhow::Result;

const INDENT: &str = "    ";
const GROUP_DEPTH: usize = 2;

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
        let mut entries: Vec<(Vec<String>, String, String)> = category.files
            .iter()
            .map(|path| {
                let key = path.to_str().unwrap().replace('\\', "/");
                (module_path(path), constant_name(path), key)
            })
            .collect();

        entries.sort();
        disambiguate(&mut entries);

        write_module(&mut source, category.module, category.handle, 0, &entries);
        source.push('\n');
    }

    write(generated_dir.join("resources.rs"), source)?;

    Ok(())
}

fn write_module(source: &mut String, name: &str, handle: &str, depth: usize, entries: &[(Vec<String>, String, String)]) {
    let indent = INDENT.repeat(depth);

    source.push_str(&format!("{}pub mod {} {{\n", indent, name));
    source.push_str(&format!("{}{}use super::{};\n", indent, INDENT, handle));

    let split = entries.partition_point(|(modules, _, _)| modules.len() == depth);
    let (constants, nested) = entries.split_at(split);

    if !constants.is_empty() {
        source.push('\n');
    }

    for (_, constant, key) in constants {
        source.push_str(&format!(
            "{}{}pub const {}: {} = {}(\"{}\");\n",
            indent, INDENT, constant, handle, handle, key,
        ));
    }

    for group in nested.chunk_by(|a, b| a.0[depth] == b.0[depth]) {
        source.push('\n');
        write_module(source, &group[0].0[depth], handle, depth + 1, group);
    }

    source.push_str(&format!("{}}}\n", indent));
}

fn module_path(path: &Path) -> Vec<String> {
    path.parent()
        .unwrap()
        .iter()
        .skip(GROUP_DEPTH)
        .map(|directory| module_name(directory.to_str().unwrap()))
        .collect()
}

fn constant_name(path: &Path) -> String {
    let stem = path.file_stem().unwrap().to_str().unwrap();

    sanitize(stem, char::to_ascii_uppercase)
}

fn module_name(directory: &str) -> String {
    sanitize(directory, char::to_ascii_lowercase)
}

fn sanitize(value: &str, case: fn(&char) -> char) -> String {
    let mut result = String::with_capacity(value.len());

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            result.push(case(&character));
        } else {
            result.push('_');
        }
    }

    if result.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(true) {
        result.insert(0, '_');
    }

    result
}

fn disambiguate(entries: &mut [(Vec<String>, String, String)]) {
    let mut counts: HashMap<(Vec<String>, String), usize> = HashMap::new();
    for (modules, name, _) in entries.iter() {
        *counts.entry((modules.clone(), name.clone())).or_insert(0) += 1;
    }

    let mut seen: HashMap<(Vec<String>, String), usize> = HashMap::new();
    for (modules, name, _) in entries.iter_mut() {
        let identity = (modules.clone(), name.clone());
        if counts[&identity] > 1 {
            let index = seen.entry(identity).or_insert(0);
            let suffixed = format!("{}_{}", name, index);
            *index += 1;
            *name = suffixed;
        }
    }
}
