pub enum PlayMode {
    Loop,
    Once { next: u16 },
    OnceCancellable { next: u16 },
}