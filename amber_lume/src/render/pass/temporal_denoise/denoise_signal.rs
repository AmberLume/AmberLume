#[derive(Copy, Clone)]
pub enum DenoiseSignal {
    Ao { rt_mode: bool },
    Shadow { colored: bool },
}

impl DenoiseSignal {
    pub fn is_colored(self) -> bool {
        match self {
            Self::Ao { .. } => false,
            Self::Shadow { colored } => colored,
        }
    }
}
