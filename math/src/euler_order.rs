#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EulerOrder {
    Xyz,
    Yxz,
}

impl EulerOrder {
    #[inline]
    pub(crate) fn glam(self) -> glam::EulerRot {
        match self {
            EulerOrder::Xyz => glam::EulerRot::XYZ,
            EulerOrder::Yxz => glam::EulerRot::YXZ,
        }
    }
}
