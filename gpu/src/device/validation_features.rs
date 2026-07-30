use ash::vk::ValidationFeatureEnableEXT;

#[derive(Copy, Clone)]
pub enum ValidationFeatures {
    Synchronization,
    BestPractices,
    GpuAssisted,
}

impl ValidationFeatures {
    pub fn as_vulkan_feature(&self) -> ValidationFeatureEnableEXT {
        match self {
            Self::Synchronization => ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION,
            Self::BestPractices => ValidationFeatureEnableEXT::BEST_PRACTICES,
            Self::GpuAssisted => ValidationFeatureEnableEXT::GPU_ASSISTED,
        }
    }
}
