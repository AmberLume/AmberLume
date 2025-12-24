pub fn calculate_aabb(vertices: &[[f32; 3]]) -> [f32; 6] {
    if vertices.is_empty() {
        return [0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    }

    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut min_z = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut max_z = f32::NEG_INFINITY;

    for vertex in vertices {
        min_x = min_x.min(vertex[0]);
        min_y = min_y.min(vertex[1]);
        min_z = min_z.min(vertex[2]);

        max_x = max_x.max(vertex[0]);
        max_y = max_y.max(vertex[1]);
        max_z = max_z.max(vertex[2]);
    }

    [min_x, min_y, min_z, max_x, max_y, max_z]
}

pub fn calculate_global_aabb(aabbs: &[[f32; 6]]) -> [f32; 6] {
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
