use crate::format::color_space::ColorSpace;
use crate::format::format::Format;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentFormat {
    pub format: Format,
    pub color_space: ColorSpace,
}
