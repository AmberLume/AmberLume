use crate::assembler::adapter::adapter::ResourceAdapter;
use crate::assembler::adapter::material_adapter::{MaterialAdapter, MaterialResource};
use crate::data::common::primitive_data::PrimitiveData;
use anyhow::Result;
use anyhow::bail;
use bytemuck::cast_slice;
use gltf::accessor::DataType;
use gltf::buffer::Data;
use gltf::{Accessor, Primitive, Semantic};
use meshopt::generate_vertex_remap;

pub struct PrimitiveAdapter {
    material_adapter: MaterialAdapter,
}

impl PrimitiveAdapter {
    pub fn create(material_adapter: MaterialAdapter) -> Self {
        Self { material_adapter }
    }

    fn extract_vec3_f32(&self, accessor: &Accessor, buffers: &[Data]) -> Vec<[f32; 3]> {
        let view = accessor.view().unwrap();
        let index = view.buffer().index();
        let buffer = &buffers[index];
        let start = view.offset() + accessor.offset();
        let stride = view.stride().unwrap_or(accessor.size());

        if stride == 12 {
            let end = start + accessor.count() * 12;
            let floats: &[f32] = cast_slice(&buffer[start..end]);

            floats
                .chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect::<Vec<[f32; 3]>>()
        } else {
            (0..accessor.count())
                .map(|index| {
                    let offset = start + index * stride;
                    let floats: &[f32] = cast_slice(&buffer[offset..offset + 12]);

                    [floats[0], floats[1], floats[2]]
                })
                .collect()
        }
    }

    fn generate_smooth_normals(indices: &[u32], positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
        let (unique_count, remap) = generate_vertex_remap(&positions, Some(indices));

        let mut unique_normals = vec![[0.0f32; 3]; unique_count];

        for triangle in indices.chunks(3) {
            let i0 = triangle[0] as usize;
            let i1 = triangle[1] as usize;
            let i2 = triangle[2] as usize;

            let v0 = positions[i0];
            let v1 = positions[i1];
            let v2 = positions[i2];

            let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

            let face_normal = [
                edge1[1] * edge2[2] - edge1[2] * edge2[1],
                edge1[2] * edge2[0] - edge1[0] * edge2[2],
                edge1[0] * edge2[1] - edge1[1] * edge2[0],
            ];

            let unique_i0 = remap[i0] as usize;
            let unique_i1 = remap[i1] as usize;
            let unique_i2 = remap[i2] as usize;

            unique_normals[unique_i0][0] += face_normal[0];
            unique_normals[unique_i0][1] += face_normal[1];
            unique_normals[unique_i0][2] += face_normal[2];

            unique_normals[unique_i1][0] += face_normal[0];
            unique_normals[unique_i1][1] += face_normal[1];
            unique_normals[unique_i1][2] += face_normal[2];

            unique_normals[unique_i2][0] += face_normal[0];
            unique_normals[unique_i2][1] += face_normal[1];
            unique_normals[unique_i2][2] += face_normal[2];
        }

        for normal in unique_normals.iter_mut() {
            let length =
                (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            if length > 0.0 {
                normal[0] /= length;
                normal[1] /= length;
                normal[2] /= length;
            } else {
                normal[0] = 0.0;
                normal[1] = 1.0;
                normal[2] = 0.0;
            }
        }

        (0..positions.len())
            .map(|i| unique_normals[remap[i] as usize])
            .collect()
    }
}

pub struct PrimitiveResource<'a> {
    pub primitive: Primitive<'a>,

    pub buffers: &'a [Data],
}

impl ResourceAdapter for PrimitiveAdapter {
    type Input<'a> = PrimitiveResource<'a>;
    type Output = PrimitiveData;

    fn adapt<'a>(&mut self, input: &Self::Input<'a>) -> Result<Self::Output> {
        println!("Collecting primitive #{}...", input.primitive.index());

        let indices = if let Some(accessor) = input.primitive.indices() {
            let view = accessor.view().unwrap();
            let index = view.buffer().index();
            let buffer = &input.buffers[index];
            let start = view.offset() + accessor.offset();
            let end = start + (accessor.count() * accessor.size());
            let data = &buffer[start..end];

            match accessor.data_type() {
                DataType::U16 => {
                    let indices_u16: &[u16] = cast_slice(data);
                    indices_u16.iter().map(|&i| i as u32).collect()
                }
                DataType::U32 => cast_slice::<u8, u32>(data).to_vec(),
                _ => bail!("Unsupported data type"),
            }
        } else {
            bail!("Accessor not found");
        };

        let vertices = if let Some(accessor) = input.primitive.get(&Semantic::Positions) {
            self.extract_vec3_f32(&accessor, &input.buffers)
        } else {
            bail!("Accessor for positions not found");
        };

        let normals = Self::generate_smooth_normals(&indices, &vertices);

        let material_data = self.material_adapter.adapt(&MaterialResource {
            material: input.primitive.material(),

            buffers: input.buffers,
        })?;

        Ok(Self::Output {
            material_data,

            indices,
            vertices,
            normals,
        })
    }
}
