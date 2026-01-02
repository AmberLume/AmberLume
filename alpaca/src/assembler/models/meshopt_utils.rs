use crate::data::common::model_data::ModelData;
use crate::data::common::primitive_data::PrimitiveData;
use anyhow::Result;
use meshopt::generate_vertex_remap;

pub fn optimize_model(model_data: &mut ModelData) -> Result<()> {
    for mesh_data in &mut model_data.meshes {
        for primitive_data in &mut mesh_data.primitives {
            optimize_primitive(primitive_data)?;
        }
    }

    Ok(())
}

fn optimize_primitive(primitive: &mut PrimitiveData) -> Result<()> {
    if primitive.indices.is_empty() {
        println!("Primitive indices are empty!");
        return Ok(());
    }

    let vertex_count = primitive.positions.len();

    let vertices = (0..vertex_count).map(|index| {
        (primitive.positions[index], primitive.normals[index], primitive.uv[index])
    }).collect::<Vec<_>>();

    let (unique_count, remap) = generate_vertex_remap(&vertices, Some(&primitive.indices));

    let optimized_indices: Vec<u32> = primitive
        .indices
        .iter()
        .map(|&index| remap[index as usize])
        .collect();

    let mut optimized_vertices = vec![[0.0f32; 3]; unique_count];
    let mut optimized_normals = vec![[0.0f32; 3]; unique_count];
    let mut optimized_uv = vec![[0.0f32; 2]; unique_count];

    for (old_index, &new_index) in remap.iter().enumerate() {
        optimized_vertices[new_index as usize] = primitive.positions[old_index];
        optimized_normals[new_index as usize] = primitive.normals[old_index];
        optimized_uv[new_index as usize] = [primitive.uv[old_index][0], 1.0 - primitive.uv[old_index][1]];
    }

    primitive.indices = optimized_indices;
    primitive.positions = optimized_vertices;
    primitive.normals = optimized_normals;
    primitive.uv = optimized_uv;

    Ok(())
}
