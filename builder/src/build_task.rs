use std::path::PathBuf;
use crate::build_target::BuildTarget;
use amber_lume::data::resource_key::ResourceKey;

pub enum BuildTask {
    RouteTarget(RouteTarget),
    CompileShader(ShaderTask),
    ExtractScenes(ExtractAssetsTask),
    ConvertKTX2(ConvertKTX2Task),
    WriteFile(WriteFileTask),
}

impl BuildTask {
    pub fn archive(
        build_target: &BuildTarget,
        resource_key: &ResourceKey,
        data: Vec<u8>,
    ) -> Self {
        let target_path = build_target.destination.join(&resource_key.value);

        Self::WriteFile(WriteFileTask {
            target_path,

            data,
        })
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub enum BuildTaskKey {
    RouteTarget { build_target: BuildTarget },
    CompileShader { build_target: BuildTarget },
    ExtractScenes { build_target: BuildTarget },
    ConvertKTX2 { build_target: BuildTarget, resource_key: String, source: PathBuf },
    WriteFile { target_path: PathBuf },
}

impl BuildTaskKey {
    pub fn from_task(build_task: &BuildTask) -> Self {
        match build_task {
            BuildTask::RouteTarget(task) => BuildTaskKey::RouteTarget {
                build_target: task.build_target.clone(),
            },
            BuildTask::CompileShader(task) => BuildTaskKey::CompileShader {
                build_target: task.build_target.clone(),
            },
            BuildTask::ExtractScenes(task) => BuildTaskKey::ExtractScenes {
                build_target: task.build_target.clone(),
            },
            BuildTask::ConvertKTX2(task) => BuildTaskKey::ConvertKTX2 {
                build_target: task.build_target.clone(),
                resource_key: task.resource_key.clone(),
                source: task.source.clone(),
            },
            BuildTask::WriteFile(task) => BuildTaskKey::WriteFile {
                target_path: task.target_path.clone(),
            },
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum BuildTaskStatus {
    Started,
    Completed,
    Failed,
}

pub struct RouteTarget {
    pub build_target: BuildTarget,
}

pub struct ShaderTask {
    pub build_target: BuildTarget,
}

pub struct ExtractAssetsTask {
    pub build_target: BuildTarget,
}

pub enum TextureType {
    Color,
    Normal,
    OcclusionRoughnessMetallic,
}

pub struct ConvertKTX2Task {
    pub build_target: BuildTarget,

    pub resource_key: String,

    pub source: PathBuf,

    pub texture_type: TextureType,
}

pub struct WriteFileTask {
    pub target_path: PathBuf,

    pub data: Vec<u8>,
}
