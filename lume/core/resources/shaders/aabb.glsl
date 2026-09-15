#ifndef AABB_GLSL
#define AABB_GLSL

const float FLT_MAX = 3.402823466e+38;

void transform_aabb(mat4 transform, vec3 local_min, vec3 local_max, out vec3 transformed_min, out vec3 transformed_max) {
    vec3 local_center = (local_min + local_max) * 0.5;
    vec3 local_extents = (local_max - local_min) * 0.5;

    vec3 center = (transform * vec4(local_center, 1.0)).xyz;

    mat3 abs_transform = mat3(
        abs(transform[0].xyz),
        abs(transform[1].xyz),
        abs(transform[2].xyz)
    );
    vec3 extents = abs_transform * local_extents;

    transformed_min = center - extents;
    transformed_max = center + extents;
}

#endif
