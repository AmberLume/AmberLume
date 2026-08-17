use crate::chunk::{ChunkCoordinate, ChunkGeometry};
use crate::source::terrain_source::TerrainSource;
use anyhow::{Result, ensure};

pub struct ProceduralTerrainSource {
    seed: u32,
    amplitude: f32,
    feature_size: f32,
    octaves: u32,
}

impl ProceduralTerrainSource {
    pub const DEFAULT_SEED: u32 = 0x9E3779B9;
    pub const DEFAULT_AMPLITUDE: f32 = 4.0;
    pub const DEFAULT_FEATURE_SIZE: f32 = 15.0;
    pub const DEFAULT_OCTAVES: u32 = 4;

    pub const LACUNARITY: f32 = 2.0;
    pub const GAIN: f32 = 0.5;

    pub fn create() -> Self {
        Self {
            seed: Self::DEFAULT_SEED,
            amplitude: Self::DEFAULT_AMPLITUDE,
            feature_size: Self::DEFAULT_FEATURE_SIZE,
            octaves: Self::DEFAULT_OCTAVES,
        }
    }

    pub fn with_shape(mut self, amplitude: f32, feature_size: f32, octaves: u32) -> Self {
        self.amplitude = amplitude;
        self.feature_size = feature_size;
        self.octaves = octaves;

        self
    }

    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        let mut frequency = 1.0 / self.feature_size;
        let mut weight = 1.0;
        let mut total = 0.0;
        let mut normalization = 0.0;

        for octave in 0..self.octaves {
            total += weight * self.noise(x * frequency, z * frequency, octave);
            normalization += weight;

            frequency *= Self::LACUNARITY;
            weight *= Self::GAIN;
        }

        total / normalization * self.amplitude
    }

    fn noise(&self, x: f32, z: f32, octave: u32) -> f32 {
        let cell_x = x.floor();
        let cell_z = z.floor();

        let fraction_x = x - cell_x;
        let fraction_z = z - cell_z;

        let weight_x = fraction_x * fraction_x * (3.0 - 2.0 * fraction_x);
        let weight_z = fraction_z * fraction_z * (3.0 - 2.0 * fraction_z);

        let corner_00 = self.corner(cell_x as i32, cell_z as i32, octave);
        let corner_10 = self.corner(cell_x as i32 + 1, cell_z as i32, octave);
        let corner_01 = self.corner(cell_x as i32, cell_z as i32 + 1, octave);
        let corner_11 = self.corner(cell_x as i32 + 1, cell_z as i32 + 1, octave);

        let near = corner_00 + (corner_10 - corner_00) * weight_x;
        let far = corner_01 + (corner_11 - corner_01) * weight_x;

        near + (far - near) * weight_z
    }

    fn corner(&self, x: i32, z: i32, octave: u32) -> f32 {
        let mut value = (x as u32)
            .wrapping_mul(0x85EBCA6B)
            .wrapping_add((z as u32).wrapping_mul(0xC2B2AE35))
            .wrapping_add(octave.wrapping_mul(0x27D4EB2F))
            .wrapping_add(self.seed);

        value ^= value >> 15;
        value = value.wrapping_mul(0x2545F491);
        value ^= value >> 13;

        (value as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

impl TerrainSource for ProceduralTerrainSource {
    fn fill(&self, coordinate: ChunkCoordinate, heights: &mut [f32]) -> Result<()> {
        ensure!(
            heights.len() == ChunkGeometry::WINDOW_LENGTH,
            "terrain source expects {} heights, got {}",
            ChunkGeometry::WINDOW_LENGTH,
            heights.len()
        );

        let border = ChunkGeometry::BORDER as i32;
        let stride = ChunkGeometry::WINDOW_STRIDE as i32;

        for row in -border..stride - border {
            for column in -border..stride - border {
                let position = ChunkGeometry::node_world_position(coordinate, column, row);

                heights[ChunkGeometry::window_index(column, row)] =
                    self.height_at(position.x, position.z);
            }
        }

        Ok(())
    }
}
