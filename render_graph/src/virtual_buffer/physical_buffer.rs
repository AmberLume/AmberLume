use gpu::BufferRange;

#[derive(Clone, Copy)]
pub struct PhysicalBuffer {
    pub range: BufferRange,
}

impl PhysicalBuffer {
    pub fn create(range: BufferRange) -> Self {
        Self {
            range,
        }
    }
}
