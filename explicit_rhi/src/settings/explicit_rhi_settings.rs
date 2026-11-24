use crate::settings::validation::Validation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExplicitRhiSettings {
    pub validation: Validation,
}
