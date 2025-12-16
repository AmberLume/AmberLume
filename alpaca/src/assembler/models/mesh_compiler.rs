use crate::assembler::resource_compiler::ResourceCompiler;
use crate::data::flatbuffers::mesh_generated::alpaca::{Mesh, MeshArgs, Vec3};
use anyhow::{Result, bail};
use flatbuffers::FlatBufferBuilder;
use gltf::accessor::DataType;
use gltf::buffer::Data;
use gltf::{Primitive, Semantic};
use meshopt::{
    VertexDataAdapter, optimize_overdraw_in_place, optimize_vertex_cache, optimize_vertex_fetch,
};
use std::slice::from_raw_parts;

pub struct MeshCompiler {}

impl MeshCompiler {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn optimize_geometry(
        &self,
        primitive: &Primitive,
        buffers: &[Data],
    ) -> Result<(Vec<u32>, Vec<[f32; 3]>)> {
        let indices = if let Some(accessor) = primitive.indices() {
            let view = accessor.view().unwrap();
            let index = view.buffer().index();
            let buffer = &buffers[index];
            let start = view.offset() + accessor.offset();
            let end = start + (accessor.count() * accessor.size());

            match accessor.data_type() {
                DataType::U16 => {
                    let data = &buffer[start..end];
                    data.chunks_exact(2)
                        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]) as u32)
                        .collect::<Vec<u32>>()
                }
                DataType::U32 => {
                    let data = &buffer[start..end];
                    data.chunks_exact(4)
                        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                        .collect::<Vec<u32>>()
                }
                _ => bail!("Unsupported data type"),
            }
        } else {
            bail!("Accessor not found");
        };

        let positions = if let Some(accessor) = primitive.get(&Semantic::Positions) {
            let view = accessor.view().unwrap();
            let index = view.buffer().index();
            let buffer = &buffers[index];
            let start = view.offset() + accessor.offset();
            let stride = view.stride().unwrap_or(accessor.size());

            (0..accessor.count())
                .map(|i| {
                    let offset = start + i * stride;
                    let data = &buffer[offset..offset + 12]; // f32 * 3

                    [
                        f32::from_le_bytes([data[0], data[1], data[2], data[3]]),
                        f32::from_le_bytes([data[4], data[5], data[6], data[7]]),
                        f32::from_le_bytes([data[8], data[9], data[10], data[11]]),
                    ]
                })
                .collect::<Vec<[f32; 3]>>()
        } else {
            bail!("Accessor for positions not found");
        };

        let vertex_data: &[u8] = unsafe {
            from_raw_parts(
                positions.as_ptr() as *const u8,
                positions.len() * size_of::<[f32; 3]>(),
            )
        };

        let vertices = VertexDataAdapter::new(vertex_data, size_of::<[f32; 3]>(), 0)?;

        let mut optimized_indices = optimize_vertex_cache(&indices, positions.len());

        optimize_overdraw_in_place(&mut optimized_indices, &vertices, 1.05);

        let optimized_positions = optimize_vertex_fetch(&mut optimized_indices, &positions);

        println!(
            "Indices count: {}, Vertex count: {} -> {}",
            indices.len(),
            positions.len(),
            optimized_positions.len()
        );

        Ok((optimized_indices, optimized_positions))
    }

    pub fn calculate_aabb(&self, positions: &[[f32; 3]]) -> [f32; 6] {
        if positions.is_empty() {
            return [0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for pos in positions {
            min_x = min_x.min(pos[0]);
            min_y = min_y.min(pos[1]);
            min_z = min_z.min(pos[2]);

            max_x = max_x.max(pos[0]);
            max_y = max_y.max(pos[1]);
            max_z = max_z.max(pos[2]);
        }

        [min_x, min_y, min_z, max_x, max_y, max_z]
    }

    pub fn calculate_global_aabb(&self, aabbs: &[[f32; 6]]) -> [f32; 6] {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for bounds in aabbs {
            min_x = min_x.min(bounds[0]);
            min_y = min_y.min(bounds[1]);
            min_z = min_z.min(bounds[2]);

            max_x = max_x.max(bounds[3]);
            max_y = max_y.max(bounds[4]);
            max_z = max_z.max(bounds[5]);
        }

        [min_x, min_y, min_z, max_x, max_y, max_z]
    }
}

pub struct MeshResource {
    pub indices: Vec<u32>,
    pub vertices: Vec<[f32; 3]>,
}

impl ResourceCompiler for MeshCompiler {
    type Input = MeshResource;

    fn compile(&self, input: Self::Input) -> Result<Vec<u8>> {
        let mut builder = FlatBufferBuilder::new();

        let indices_offset = builder.create_vector(&input.indices);

        let positions_vec: Vec<Vec3> = input
            .vertices
            .iter()
            .map(|p| Vec3::new(p[0], p[1], p[2]))
            .collect();
        let positions_offset = builder.create_vector(&positions_vec);

        let mesh = Mesh::create(
            &mut builder,
            &MeshArgs {
                indices: Some(indices_offset),
                positions: Some(positions_offset),
            },
        );

        builder.finish(mesh, None);

        Ok(builder.finished_data().to_vec())
    }
}
