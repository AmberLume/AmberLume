#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum HardwareKeyState {
    #[default]
    Up,
    JustPressed,
    Held,
    JustReleased,
}
