use render::PresentMode;

pub trait PresentModeChoice {
    const OPTIONS: &'static [&'static str];

    fn from_index(index: usize) -> Self;

    fn index(self) -> usize;
}

impl PresentModeChoice for PresentMode {
    const OPTIONS: &'static [&'static str] = &["Immediate", "Mailbox", "FIFO"];

    fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Immediate,
            1 => Self::Mailbox,
            _ => Self::Fifo,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Immediate => 0,
            Self::Mailbox => 1,
            Self::Fifo => 2,
        }
    }
}
