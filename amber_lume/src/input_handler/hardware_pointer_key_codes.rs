use std::mem::transmute;

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HardwarePointerKeyCodes {
    Left,
    Right,
    Middle,
    Forward,
    Backward,
    Count,
}

impl HardwarePointerKeyCodes {
    pub fn from_index(i: u32) -> Self {
        unsafe { transmute(i) }
    }
}
