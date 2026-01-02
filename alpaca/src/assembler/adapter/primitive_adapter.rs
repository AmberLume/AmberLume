use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::assembler::adapter::adapter::ResourceAdapter;
use crate::data::common::primitive_data::PrimitiveData;
use anyhow::Result;
use anyhow::bail;
use gltf::buffer::Data;
use gltf::Primitive;
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

        let positions: Vec<[f32; 3]> = if let Some(positions) = reader.read_positions() {
            positions.collect::<Vec<[f32; 3]>>()
        } else {
            bail!("Accessor for positions not found");
        };

        let uv: Vec<[f32; 2]> = if let Some(texture_coordinates) = reader.read_tex_coords(0) {
            texture_coordinates.into_f32().collect()
        } else {
            bail!("Accessor for texture coordinates not found");
        };

        let normals: Vec<[f32; 3]> = if let Some(iter) = reader.read_normals() {
            iter.collect::<Vec<[f32; 3]>>()
        } else {
            bail!("Accessor for normal not found");
        };

        let positions_count = positions.len();
        let uv_count = uv.len();
        let normals_count = normals.len();

        assert!(
            positions_count == uv_count && uv_count == normals_count,
            "Model arrays are not equals! Positions: {}, UVs: {}, normals: {}",
            positions_count, uv_count, normals_count,
        );

        let mut material_collector = self.material_collector.lock().unwrap();

        let material_id = material_collector.collect(&MaterialResource {
            material: input.primitive.material(),

            local_path: input.local_path,
            
            buffers: input.buffers,
        })?;
        
        Ok(Self::Output {
            material_id,

            indices,
            positions,
            normals,
            uv,
        })
    }
}
