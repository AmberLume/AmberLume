#[derive(Copy, Clone)]
pub enum DenoiseSignal {
    Ao { rt_mode: bool },
    Shadow,
}
