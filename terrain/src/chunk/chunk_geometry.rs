use crate::chunk::chunk_coordinate::ChunkCoordinate;
use glam::Vec3;

pub struct ChunkGeometry;

impl ChunkGeometry {
    pub const CELLS: u32 = 32;
    pub const CELL_SIZE: f32 = 1.0;
    pub const BORDER: u32 = 1;

    pub const NODES: u32 = Self::CELLS + 1;
    pub const NODE_COUNT: u32 = Self::NODES * Self::NODES;
    pub const INDEX_COUNT: u32 = Self::CELLS * Self::CELLS * 6;

    pub const OWNED_HEIGHT_COUNT: u32 = Self::CELLS * Self::CELLS;

    pub const WINDOW_STRIDE: u32 = Self::NODES + Self::BORDER * 2;
    pub const WINDOW_LENGTH: usize = (Self::WINDOW_STRIDE * Self::WINDOW_STRIDE) as usize;

    pub const LAYER_STRIDE: u32 = Self::NODES;
    pub const LAYER_LENGTH: usize = (Self::LAYER_STRIDE * Self::LAYER_STRIDE) as usize;

    pub const CHUNK_SIZE: f32 = Self::CELLS as f32 * Self::CELL_SIZE;
    pub const HALF_SIZE: f32 = Self::CHUNK_SIZE * 0.5;

    pub fn chunk_center(coordinate: ChunkCoordinate) -> Vec3 {
        Vec3::new(
            (coordinate.x as f32 + 0.5) * Self::CHUNK_SIZE,
            0.0,
            (coordinate.z as f32 + 0.5) * Self::CHUNK_SIZE,
        )
    }

    pub fn chunk_of(position: Vec3) -> ChunkCoordinate {
        ChunkCoordinate::create(
            (position.x / Self::CHUNK_SIZE).floor() as i32,
            (position.z / Self::CHUNK_SIZE).floor() as i32,
        )
    }

    pub fn node_world_position(coordinate: ChunkCoordinate, column: i32, row: i32) -> Vec3 {
        Vec3::new(
            coordinate.x as f32 * Self::CHUNK_SIZE + column as f32 * Self::CELL_SIZE,
            0.0,
            coordinate.z as f32 * Self::CHUNK_SIZE + row as f32 * Self::CELL_SIZE,
        )
    }

    pub fn node_local_position(column: i32, row: i32, height: f32) -> Vec3 {
        Vec3::new(
            column as f32 * Self::CELL_SIZE - Self::HALF_SIZE,
            height,
            row as f32 * Self::CELL_SIZE - Self::HALF_SIZE,
        )
    }

    pub fn window_index(column: i32, row: i32) -> usize {
        let border = Self::BORDER as i32;
        let stride = Self::WINDOW_STRIDE as i32;
        let clamped_column = (column + border).clamp(0, stride - 1);
        let clamped_row = (row + border).clamp(0, stride - 1);

        (clamped_row * stride + clamped_column) as usize
    }
}
