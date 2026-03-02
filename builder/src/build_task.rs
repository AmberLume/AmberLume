use std::path::PathBuf;
use std::sync::Arc;
use crate::gltf_file::GltfFile;
use crate::paths::Paths;

pub enum BuildTask {
    SeedFile(SeedFileTask),
    CompileShader(ShaderTask),
    ExtractScenes(ExtractScenesTask),
    CollectScene(CollectSceneTask),
    ExtractModelAsset(ExtractModelAssetTask),
    ConvertKTX2(ConvertKTX2Task),
    WriteFile(WriteFileTask),
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub enum BuildTaskKey {
    SeedFile { paths: Paths },
    CompileShader { paths: Paths },
    ExtractScenes { paths: Paths },
    CollectScene { name: String, paths: Paths },
    ExtractModelAsset { collection_name: String, file_name: String, paths: Paths },
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
            BuildTask::CollectScene(task) => BuildTaskKey::CollectScene {
                name: task.name.clone(),
                paths: task.paths.clone(),
            },
            BuildTask::ExtractModelAsset(task) => BuildTaskKey::ExtractModelAsset {
                collection_name: task.collection_name.clone(),
                file_name: task.file_name.clone(),
                paths: task.paths.clone(),
            },
            BuildTask::ConvertKTX2(task) => BuildTaskKey::ConvertKTX2 {
                source_path: task.source_path.clone(),
                target_path: task.target_path.clone(),
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
    pub paths: Paths,
}

pub struct ShaderTask {
    pub paths: Paths,
}

pub struct ExtractScenesTask {
    pub paths: Paths,
}

pub struct CollectSceneTask {
    pub name: String,

    pub scene_index: usize,
    pub gltf_file: Arc<GltfFile>,

    pub paths: Paths,
}

pub struct ExtractModelAssetTask {
    pub collection_name: String,
    pub file_name: String,

    pub paths: Paths,
}

pub enum TextureType {
    Color,
    Normal,
}

pub struct ConvertKTX2Task {
    pub name: String,
    
    pub source_path: PathBuf,

    pub target_path: PathBuf,
    
    pub texture_type: TextureType,
}

pub struct WriteFileTask {
    pub target_path: PathBuf,

    pub data: Vec<u8>,
}
