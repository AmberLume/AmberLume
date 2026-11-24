use anyhow::Result;
use ash::Entry;
use ash::ext::validation_features;
use ash::vk::ValidationFeatureEnableEXT;
use explicit_rhi::Validation;
use logger::warn;
use std::ffi::CStr;

pub(crate) struct ValidationLayer {
    pub features: Vec<ValidationFeatureEnableEXT>,
}

impl ValidationLayer {
    pub const NAME: &'static CStr = c"VK_LAYER_KHRONOS_validation";

    pub fn create(entry: &Entry, validation: Validation) -> Result<Option<Self>> {
        if !validation.is_enabled() {
            return Ok(None);
        }

        let layers = unsafe { entry.enumerate_instance_layer_properties()? };
        let available = layers.iter().any(|layer| layer.layer_name_as_c_str() == Ok(Self::NAME));

        if !available {
            warn!("Validation requested, but {:?} is not available", Self::NAME);

            return Ok(None);
        }

        let mut features = Vec::new();

        if validation.synchronization {
            features.push(ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION);
        }
        if validation.best_practices {
            features.push(ValidationFeatureEnableEXT::BEST_PRACTICES);
        }
        if validation.gpu_assisted {
            features.push(ValidationFeatureEnableEXT::GPU_ASSISTED);
        }

        if !features.is_empty() && !Self::supports_features(entry)? {
            warn!("Validation features requested, but {:?} does not support them", Self::NAME);

            features.clear();
        }

        Ok(Some(Self { features }))
    }

    fn supports_features(entry: &Entry) -> Result<bool> {
        let extensions = unsafe { entry.enumerate_instance_extension_properties(Some(Self::NAME))? };

        Ok(extensions
            .iter()
            .any(|extension| extension.extension_name_as_c_str() == Ok(validation_features::NAME)))
    }
}
