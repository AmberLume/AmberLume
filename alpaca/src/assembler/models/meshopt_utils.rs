use crate::data::common::model_data::ModelData;
use crate::data::common::primitive_data::PrimitiveData;
use anyhow::Result;
use bytemuck::cast_slice;
use meshopt::{
    VertexDataAdapter, optimize_overdraw_in_place, optimize_vertex_cache, optimize_vertex_fetch,
};

pub fn optimize_model(model_data: &mut ModelData) -> Result<()> {
    for mesh_data in &mut model_data.meshes {
        for primitive_data in &mut mesh_data.primitives {
            optimize_primitive(primitive_data)?;
        }
    }

    Ok(())
}

fn optimize_primitive(primitive: &mut PrimitiveData) -> Result<()> {
    if primitive.vertices.is_empty() || primitive.indices.is_empty() {
        println!("Primitive vertices are empty!");
        return Ok(());
    }

    let vertices_count = primitive.vertices.len();

    let vertices =
        VertexDataAdapter::new(cast_slice(&primitive.vertices), size_of::<[f32; 3]>(), 0)?;

    let mut optimized_indices = optimize_vertex_cache(&primitive.indices, vertices_count);

    optimize_overdraw_in_place(&mut optimized_indices, &vertices, 1.05);

    let optimized_vertices = optimize_vertex_fetch(&mut optimized_indices, &primitive.vertices);

    println!(
        "Indices count: {}, Vertex count: {} -> {}",
        primitive.indices.len(),
        vertices_count,
        optimized_vertices.len()
    );

    primitive.indices = optimized_indices;
    primitive.vertices = optimized_vertices;

    Ok(())
}
