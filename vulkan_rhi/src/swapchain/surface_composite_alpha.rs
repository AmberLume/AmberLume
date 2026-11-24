use anyhow::{Result, anyhow};
use ash::vk::{CompositeAlphaFlagsKHR, SurfaceCapabilitiesKHR};
use explicit_rhi::CompositeAlpha;

pub(crate) struct SurfaceCompositeAlpha;

impl SurfaceCompositeAlpha {
    pub fn select(
        capabilities: &SurfaceCapabilitiesKHR,
        composite_alphas: &[CompositeAlpha],
    ) -> Result<CompositeAlphaFlagsKHR> {
        composite_alphas
            .iter()
            .map(|&composite_alpha| Self::vulkan_composite_alpha(composite_alpha))
            .find(|&composite_alpha| capabilities.supported_composite_alpha.contains(composite_alpha))
            .ok_or_else(|| {
                anyhow!(
                    "None of the composite alphas {:?} is supported by the surface, supported: {:?}",
                    composite_alphas,
                    capabilities.supported_composite_alpha,
                )
            })
    }

    fn vulkan_composite_alpha(composite_alpha: CompositeAlpha) -> CompositeAlphaFlagsKHR {
        match composite_alpha {
            CompositeAlpha::Opaque => CompositeAlphaFlagsKHR::OPAQUE,
            CompositeAlpha::PreMultiplied => CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
            CompositeAlpha::PostMultiplied => CompositeAlphaFlagsKHR::POST_MULTIPLIED,
            CompositeAlpha::Inherit => CompositeAlphaFlagsKHR::INHERIT,
        }
    }
}
