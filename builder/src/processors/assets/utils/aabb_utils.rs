pub fn calculate_aabb<I>(vertices: I) -> [f32; 6]
where
    I: IntoIterator<Item = [f32; 3]>
{
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];

    for [x, y, z] in vertices {
        min[0] = min[0].min(x);
        min[1] = min[1].min(y);
        min[2] = min[2].min(z);
        max[0] = max[0].max(x);
        max[1] = max[1].max(y);
        max[2] = max[2].max(z);
    }

    [min[0], min[1], min[2], max[0], max[1], max[2]]
}

pub fn calculate_global_aabb<I>(aabbs: I) -> [f32; 6]
where
    I: IntoIterator<Item = [f32; 6]>
{
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];

    for bounds in aabbs {
        min[0] = min[0].min(bounds[0]);
        min[1] = min[1].min(bounds[1]);
        min[2] = min[2].min(bounds[2]);
        max[0] = max[0].max(bounds[3]);
        max[1] = max[1].max(bounds[4]);
        max[2] = max[2].max(bounds[5]);
    }

    [min[0], min[1], min[2], max[0], max[1], max[2]]
}
