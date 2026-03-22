use std::path::PathBuf;
use crate::paths::AlpacaPaths;

pub enum BuildTask {
    SeedFile(SeedFileTask),
    CompileShader(ShaderTask),
    ExtractScenes(ExtractAssetsTask),
    ConvertKTX2(ConvertKTX2Task),
    WriteFile(WriteFileTask),
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub enum BuildTaskKey {
    SeedFile { paths: AlpacaPaths },
    CompileShader { paths: AlpacaPaths },
    ExtractScenes { paths: AlpacaPaths },
    ConvertKTX2 { source_path: PathBuf, target_path: PathBuf },
    WriteFile { target_path: PathBuf },
}

impl BuildTaskKey {
    pub fn from_task(build_task: &BuildTask) -> Self {
        match build_task {
            BuildTask::SeedFile(task) => BuildTaskKey::SeedFile {
                paths: task.paths.clone(),
            },
            BuildTask::CompileShader(task) => BuildTaskKey::CompileShader {
                paths: task.paths.clone(),
            },
            BuildTask::ExtractScenes(task) => BuildTaskKey::ExtractScenes {
                paths: task.paths.clone(),
            },
            BuildTask::ConvertKTX2(task) => BuildTaskKey::ConvertKTX2 {
                source_path: task.source.clone(),
                target_path: task.target.clone(),
            },
            BuildTask::WriteFile(task) => BuildTaskKey::WriteFile {
                target_path: task.target_path.clone(),
            },
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum BuildTaskStatis {
    Started,
    Completed,
    Failed,
}

pub struct SeedFileTask {
    pub paths: AlpacaPaths,
}

pub struct ShaderTask {
    pub paths: AlpacaPaths,
}

pub struct ExtractAssetsTask {
    pub paths: AlpacaPaths,
}

pub enum TextureType {
    Color,
    Normal,
    OcclusionRoughnessMetalic,
}

pub struct ConvertKTX2Task {
    pub name: String,
    
    pub source: PathBuf,

    pub target: PathBuf,
    
    pub texture_type: TextureType,
}

pub struct WriteFileTask {
    pub target_path: PathBuf,

    pub data: Vec<u8>,
}
