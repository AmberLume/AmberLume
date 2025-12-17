use crate::data::common::model_data::ModelData;
use crate::data::common::primitive_data::PrimitiveData;

pub fn fill_aabbs(model_data: &mut ModelData) {
    let mut meshes_aabb = Vec::with_capacity(model_data.meshes.len());

    for mesh_data in &mut model_data.meshes {
        let mut primitives_aabb = Vec::with_capacity(mesh_data.primitives.len());

        for primitive_data in &mesh_data.primitives {
            let primitive_aabb = calculate_aabb(&primitive_data);

            primitives_aabb.push(primitive_aabb);
        }

        mesh_data.bounds = calculate_global_aabb(&primitives_aabb);
        meshes_aabb.push(mesh_data.bounds);
    }

    model_data.bounds = calculate_global_aabb(&meshes_aabb);
}

fn calculate_aabb(primitive_data: &PrimitiveData) -> [f32; 6] {
    if primitive_data.vertices.is_empty() {
        return [0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    }

    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut min_z = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut max_z = f32::NEG_INFINITY;

    for vertex in &primitive_data.vertices {
        min_x = min_x.min(vertex[0]);
        min_y = min_y.min(vertex[1]);
        min_z = min_z.min(vertex[2]);

        max_x = max_x.max(vertex[0]);
        max_y = max_y.max(vertex[1]);
        max_z = max_z.max(vertex[2]);
    }

    [min_x, min_y, min_z, max_x, max_y, max_z]
}

fn calculate_global_aabb(aabbs: &[[f32; 6]]) -> [f32; 6] {
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
