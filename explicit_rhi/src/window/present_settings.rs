use crate::format::color_space::ColorSpace;
use crate::format::format::Format;
use crate::image::image_usage::ImageUsage;
use crate::window::composite_alpha::CompositeAlpha;
use crate::window::present_format::PresentFormat;
use crate::window::present_mode::PresentMode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresentSettings {
    pub formats: Vec<PresentFormat>,
    pub present_modes: Vec<PresentMode>,
    pub composite_alphas: Vec<CompositeAlpha>,

    pub image_count: u32,
    pub usage: ImageUsage,
}

impl Default for PresentSettings {
    fn default() -> Self {
        let formats = vec![
            PresentFormat {
                format: Format::Bgra8UnormSrgb,
                color_space: ColorSpace::SrgbNonlinear,
            },
            PresentFormat {
                format: Format::Rgba8UnormSrgb,
                color_space: ColorSpace::SrgbNonlinear,
            },
        ];
        let present_modes = vec![PresentMode::Fifo];
        let composite_alphas = vec![
            CompositeAlpha::Opaque,
            CompositeAlpha::Inherit,
            CompositeAlpha::PreMultiplied,
            CompositeAlpha::PostMultiplied,
        ];

        let image_count = 3;
        let usage = ImageUsage {
            color_attachment: true,
        };

        Self {
            formats,
            present_modes,
            composite_alphas,

            image_count,
            usage,
        }
    }
}
