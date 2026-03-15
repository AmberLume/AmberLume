pub trait Statistics {
    type Snapshot: Copy + Smooth;

    fn snapshot(&self) -> Self::Snapshot;
}

pub trait Smooth {
    fn smooth(&self, other: &Self, alpha: f32) -> Self;
}
