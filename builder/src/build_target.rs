use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct BuildTarget {
    pub entry: PathBuf,

    pub name: String,
    pub extension: String,

    pub relative: PathBuf,

    pub destination: PathBuf,
}

impl BuildTarget {
    pub fn new(source: &Path, destination: &Path, entry: &Path) -> Option<Self> {
        let entry = entry.to_path_buf();
        let source = source.to_path_buf();

        let name = entry.file_stem()?.to_str()?.to_string();
        let extension = entry.extension()?.to_str()?.to_string();

        let relative = entry.strip_prefix(&source).ok()?.parent()?.to_path_buf();

        Some(BuildTarget {
            entry,

            name,
            extension,

            relative,

            destination: destination.to_path_buf(),
        })
    }

    pub fn relative_full(&self) -> PathBuf {
        self.relative.join(&self.name).with_extension(&self.extension)
    }
}

pub fn targets_from(
    source: &Path,
    source_dirs: &[&str],
    destination: &Path,
    extensions: &[&str],
) -> Vec<BuildTarget> {
    source_dirs.iter()
        .flat_map(|source_dir| {
            WalkDir::new(&source.join(source_dir))
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().is_file())
                .filter(|entry| {
                    entry.path()
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| extensions.contains(&e))
                        .unwrap_or(false)
                })
                .filter_map(|entry| {
                    BuildTarget::new(source, destination, &entry.path())
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}
