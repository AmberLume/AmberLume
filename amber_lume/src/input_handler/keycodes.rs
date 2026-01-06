#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keycode {
    Esc = 0,

    F1 = 1, F2 = 2, F3 = 3, F4 = 4, F5 = 5, F6 = 6, F7 = 7, F8 = 8, F9 = 9, F10 = 10, F11 = 11, F12 = 12,

    Q = 13, W = 14, E = 15, R = 16,
    A = 17, S = 18, D = 19, F = 20,
    Z = 21, X = 22, C = 23, V = 24,

    Space = 25,

    ArrowUp = 26, ArrowLeft = 27, ArrowDown = 28, ArrowRight = 29,

    Count = 30,
}
