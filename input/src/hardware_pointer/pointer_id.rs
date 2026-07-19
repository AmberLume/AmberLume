#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct PointerId {
    pub id: i32,
}

impl PointerId {
    pub fn new(id: i32) -> Self {
        Self { id }
    }
}
