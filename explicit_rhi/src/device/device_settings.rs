use crate::device::validation::Validation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceSettings {
    pub validation: Validation,
}
