use crate::chunk::chunk_coordinate::ChunkCoordinate;
use crate::chunk::chunk_geometry::ChunkGeometry;
use glam::Vec3;

pub struct ChunkPayload {
    coordinate: ChunkCoordinate,

    heights: [f32; ChunkGeometry::WINDOW_LENGTH],

    minimum: f32,
    maximum: f32,
}

impl ChunkPayload {
    pub fn empty(coordinate: ChunkCoordinate) -> Self {
        Self {
            coordinate,

            heights: [0.0; ChunkGeometry::WINDOW_LENGTH],

            minimum: 0.0,
            maximum: 0.0,
        }
    }

    pub fn coordinate(&self) -> ChunkCoordinate {
        self.coordinate
    }

    pub fn heights(&self) -> &[f32] {
        &self.heights
    }

    pub fn heights_mut(&mut self) -> &mut [f32] {
        &mut self.heights
    }

    pub fn recalculate_bounds(&mut self) {
        let nodes = ChunkGeometry::NODES as i32;

        let mut minimum = f32::MAX;
        let mut maximum = f32::MIN;

        for row in 0..nodes {
            for column in 0..nodes {
                let height = self.height(column, row);

                minimum = minimum.min(height);
                maximum = maximum.max(height);
            }
        }

        self.minimum = minimum;
        self.maximum = maximum;
    }

    pub fn minimum(&self) -> f32 {
        self.minimum
    }

    pub fn maximum(&self) -> f32 {
        self.maximum
    }

    pub fn height(&self, column: i32, row: i32) -> f32 {
        self.heights[ChunkGeometry::window_index(column, row)]
    }

    pub fn normal(&self, column: i32, row: i32) -> Vec3 {
        let previous_column = self.height(column - 1, row);
        let next_column = self.height(column + 1, row);
        let previous_row = self.height(column, row - 1);
        let next_row = self.height(column, row + 1);

        Vec3::new(
            previous_column - next_column,
            2.0 * ChunkGeometry::CELL_SIZE,
            previous_row - next_row,
        )
        .normalize()
    }

    pub fn bounds(&self) -> [f32; 6] {
        [
            -ChunkGeometry::HALF_SIZE,
            self.minimum,
            -ChunkGeometry::HALF_SIZE,
            ChunkGeometry::HALF_SIZE,
            self.maximum,
            ChunkGeometry::HALF_SIZE,
        ]
    }

    pub fn collision_heights(&self) -> Vec<f32> {
        let nodes = ChunkGeometry::NODES as i32;
        let mut heights = Vec::with_capacity(ChunkGeometry::LAYER_LENGTH);

        for row in 0..nodes {
            for column in 0..nodes {
                heights.push(self.height(column, row));
            }
        }

        heights
    }
}
