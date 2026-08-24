use gpu::BufferRange;

#[derive(Clone, Copy)]
pub struct PhysicalReadback {
    pub range: BufferRange,
}

impl PhysicalReadback {
    pub fn create(range: BufferRange) -> Self {
        Self {
            range,
        }
    }
}
