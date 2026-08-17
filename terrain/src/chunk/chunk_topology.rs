use crate::chunk::chunk_geometry::ChunkGeometry;

pub struct ChunkTopology {
    indices: Vec<u32>,
}

impl ChunkTopology {
    pub fn build() -> Self {
        let nodes = ChunkGeometry::NODES;
        let mut indices = Vec::with_capacity(ChunkGeometry::INDEX_COUNT as usize);

        for row in 0..ChunkGeometry::CELLS {
            for column in 0..ChunkGeometry::CELLS {
                let near_left = row * nodes + column;
                let far_left = (row + 1) * nodes + column;
                let near_right = row * nodes + column + 1;
                let far_right = (row + 1) * nodes + column + 1;

                indices.extend_from_slice(&[
                    near_left, far_left, near_right, far_left, far_right, near_right,
                ]);
            }
        }

        Self { indices }
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }
}
