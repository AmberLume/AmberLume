use crate::chunk::chunk_coordinate::ChunkCoordinate;
use glam::Vec3;

pub struct ChunkGeometry;

impl ChunkGeometry {
    pub const CELLS: u32 = 32;
    pub const BASE_CELL_SIZE: f32 = 1.0;
    pub const BORDER: u32 = 1;

    pub const NODES: u32 = Self::CELLS + 1;
    pub const NODE_COUNT: u32 = Self::NODES * Self::NODES;
    pub const INDEX_COUNT: u32 = Self::CELLS * Self::CELLS * 6;

    pub const SIDES: u32 = 4;
    pub const PERIMETER_NODE_COUNT: u32 = Self::NODES * Self::SIDES - Self::SIDES;
    pub const EDGE_LENGTH: usize = (Self::NODES * Self::SIDES) as usize;

    pub const OWNED_HEIGHT_COUNT: u32 = Self::CELLS * Self::CELLS;

    pub const WINDOW_STRIDE: u32 = Self::NODES + Self::BORDER * 2;
    pub const WINDOW_LENGTH: usize = (Self::WINDOW_STRIDE * Self::WINDOW_STRIDE) as usize;

    pub const LAYER_STRIDE: u32 = Self::NODES;
    pub const LAYER_LENGTH: usize = (Self::LAYER_STRIDE * Self::LAYER_STRIDE) as usize;

    pub const BASE_CHUNK_SIZE: f32 = Self::CELLS as f32 * Self::BASE_CELL_SIZE;

    pub fn level_scale(level: u32) -> f32 {
        (1u32 << level) as f32
    }

    pub fn cell_size(level: u32) -> f32 {
        Self::BASE_CELL_SIZE * Self::level_scale(level)
    }

    pub fn chunk_size(level: u32) -> f32 {
        Self::BASE_CHUNK_SIZE * Self::level_scale(level)
    }

    pub fn half_size(level: u32) -> f32 {
        Self::chunk_size(level) * 0.5
    }

    pub fn distance_to(observer: Vec3, coordinate: ChunkCoordinate) -> f32 {
        let center = Self::chunk_center(coordinate);
        let half_size = Self::half_size(coordinate.level);

        let x = ((observer.x - center.x).abs() - half_size).max(0.0);
        let z = ((observer.z - center.z).abs() - half_size).max(0.0);

        (x * x + z * z).sqrt()
    }

    pub fn chunk_center(coordinate: ChunkCoordinate) -> Vec3 {
        let chunk_size = Self::chunk_size(coordinate.level);

        Vec3::new(
            (coordinate.x as f32 + 0.5) * chunk_size,
            0.0,
            (coordinate.z as f32 + 0.5) * chunk_size,
        )
    }

    pub fn chunk_of(position: Vec3, level: u32) -> ChunkCoordinate {
        let chunk_size = Self::chunk_size(level);

        ChunkCoordinate::create(
            (position.x / chunk_size).floor() as i32,
            (position.z / chunk_size).floor() as i32,
            level,
        )
    }

    pub fn node_world_position(coordinate: ChunkCoordinate, column: i32, row: i32) -> Vec3 {
        let chunk_size = Self::chunk_size(coordinate.level);
        let cell_size = Self::cell_size(coordinate.level);

        Vec3::new(
            coordinate.x as f32 * chunk_size + column as f32 * cell_size,
            0.0,
            coordinate.z as f32 * chunk_size + row as f32 * cell_size,
        )
    }

    pub fn node_local_position(level: u32, column: i32, row: i32, height: f32) -> Vec3 {
        let cell_size = Self::cell_size(level);
        let half_size = Self::half_size(level);

        Vec3::new(
            column as f32 * cell_size - half_size,
            height,
            row as f32 * cell_size - half_size,
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
