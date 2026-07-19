#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn add(&mut self, other: Point) -> &mut Point {
        self.x += other.x;
        self.y += other.y;
        
        self
    }
}
