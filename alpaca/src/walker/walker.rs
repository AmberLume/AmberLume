use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct Walker {
    input_dir: PathBuf,
}

impl Walker {
    pub fn create(input_dir: &PathBuf) -> Self {
        Self {
            input_dir: input_dir.clone(),
        }
    }

    pub fn walk<O, F>(&self, filter: O, mut block: F) -> Result<()>
    where
        F: FnMut(&Path, &String) -> Result<()>,
        O: Fn(&Path) -> bool,
    {
        for entry in WalkDir::new(&self.input_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter(|e| filter(e.path()))
        {
            let path = entry.path();

            let stripped_path = path
                .strip_prefix(&self.input_dir)?
                .to_string_lossy()
                .into_owned();

            block(path, &stripped_path)?;
        }

        Ok(())
    }
}
