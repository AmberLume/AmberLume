#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum HardwareKeyCode {
    Esc,

    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    Q, W, E, R, T, Y, U, I, O, P,
    A, S, D, F, G, H, J, K, L,
    Z, X, C, V, B, N, M,

    Space,

    ArrowUp, ArrowLeft, ArrowDown, ArrowRight,

    Count,
}
