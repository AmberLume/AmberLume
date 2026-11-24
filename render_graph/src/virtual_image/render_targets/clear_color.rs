use math::Vec4;

#[derive(Clone, Copy)]
pub enum ClearColor {
    Float(Vec4),
    Uint([u32; 4]),
}
