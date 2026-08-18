#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PresentMode {
    Immediate,
    Mailbox,
    Fifo,
}

impl PresentMode {
    pub const OPTIONS: &'static [&'static str] = &["Immediate", "Mailbox", "FIFO"];

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Immediate,
            1 => Self::Mailbox,
            _ => Self::Fifo,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Immediate => 0,
            Self::Mailbox => 1,
            Self::Fifo => 2,
        }
    }
}
