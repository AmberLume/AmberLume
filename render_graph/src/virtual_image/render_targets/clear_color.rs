#[derive(Clone, Copy)]
pub enum ClearColor {
    Float([f32; 4]),
    Uint([u32; 4]),
}
