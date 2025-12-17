use crate::data::common::primitive_data;
use anyhow::Result;
use anyhow::bail;
use gltf::accessor::DataType;
use gltf::buffer::Data;
use gltf::{Primitive, Semantic};

pub struct PrimitiveAdapter;

impl PrimitiveAdapter {
    pub fn create_from(
        primitive: &Primitive,
        buffers: &[Data],
    ) -> Result<primitive_data::PrimitiveData> {
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

        let vertices = if let Some(accessor) = primitive.get(&Semantic::Positions) {
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

        Ok(primitive_data::PrimitiveData { indices, vertices })
    }
}
