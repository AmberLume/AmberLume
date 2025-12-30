use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::assembler::adapter::adapter::ResourceAdapter;
use crate::data::common::primitive_data::PrimitiveData;
use anyhow::Result;
use anyhow::bail;
use gltf::buffer::Data;
use gltf::Primitive;
use meshopt::generate_vertex_remap;
use crate::assembler::collector::collector::ResourceCollector;
use crate::assembler::collector::material_collector::{MaterialCollector, MaterialResource};

pub struct PrimitiveAdapter {
    material_collector: Arc<Mutex<MaterialCollector>>,
}

impl PrimitiveAdapter {
    pub fn create(
        material_collector: Arc<Mutex<MaterialCollector>>,
    ) -> Self {
        Self { 
            material_collector,
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

    pub local_path: &'a Path,
    
    pub buffers: &'a [Data],
}

impl ResourceAdapter for PrimitiveAdapter {
    type Input<'a> = PrimitiveResource<'a>;
    type Output = PrimitiveData;

    fn adapt<'a>(&mut self, input: &Self::Input<'a>) -> Result<Self::Output> {
        println!("Collecting primitive #{}...", input.primitive.index());

        let reader = input.primitive.reader(|buffer| {
            Some(&input.buffers[buffer.index()])
        });

        let indices = if let Some(indices) = reader.read_indices() {
            indices.into_u32().collect::<Vec<u32>>()
        } else {
            bail!("Accessor for indices coordinates not found");
        };

        let vertices = if let Some(positions) = reader.read_positions() {
            positions.collect::<Vec<[f32; 3]>>()
        } else {
            bail!("Accessor for positions not found");
        };

        let uv = if let Some(texture_coordinates) = reader.read_tex_coords(0) {
            texture_coordinates.into_f32().collect()
        } else {
            bail!("Accessor for texture coordinates not found");
        };

        let normals = Self::generate_smooth_normals(&indices, &vertices);

        let mut material_collector = self.material_collector.lock().unwrap();

        let material_id = material_collector.collect(&MaterialResource {
            material: input.primitive.material(),

            local_path: input.local_path,
            
            buffers: input.buffers,
        })?;
        
        Ok(Self::Output {
            material_id,

            indices,
            vertices,
            normals,
            uv,
        })
    }
}
