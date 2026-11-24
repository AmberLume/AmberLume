#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositeAlpha {
    Opaque,
    PreMultiplied,
    PostMultiplied,
    Inherit,
}
