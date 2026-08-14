#ifndef SAMPLING_GLSL
#define SAMPLING_GLSL

const float GOLDEN_ANGLE = 2.399963229728653;
const float TWO_PI = 6.283185307179586;

uint hash_u32(uint x) {
    x ^= x >> 16;
    x *= 0x7feb352du;
    x ^= x >> 15;
    x *= 0x846ca68bu;
    x ^= x >> 16;
    return x;
}

float hash_unit(uint x) {
    return float(hash_u32(x)) * (1.0 / 4294967296.0);
}

vec2 vogel_disk_sample(float disk_radius, int sample_count, float seed) {
    float rotation = hash_unit(floatBitsToUint(seed) + 0x9e3779b9u) * TWO_PI;
    float radius = sqrt(seed / float(sample_count)) * disk_radius;
    float theta = seed * GOLDEN_ANGLE + rotation;
    return radius * vec2(cos(theta), sin(theta));
}

float sample_seed(ivec2 coord, uint sample_index, uint frame_number) {
    uint pixel_hash = uint(coord.x) * 0x9e3779b9u
        ^ uint(coord.y) * 0x85ebca6bu
        ^ sample_index * 0xc2b2ae35u
        ^ frame_number * 0x27d4eb2fu;

    return float(sample_index) + hash_unit(pixel_hash);
}

void orthonormal_basis(vec3 axis, out vec3 tangent, out vec3 bitangent) {
    vec3 up = abs(axis.y) < 0.999 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0);

    tangent = normalize(cross(up, axis));
    bitangent = cross(axis, tangent);
}

vec3 sun_disk_direction(vec3 to_sun, float angular_radius, ivec2 coord, uint sample_index, uint sample_count, uint frame_number) {
    vec3 tangent;
    vec3 bitangent;
    orthonormal_basis(to_sun, tangent, bitangent);

    vec2 disk = vogel_disk_sample(tan(angular_radius), int(sample_count), sample_seed(coord, sample_index, frame_number));

    return normalize(to_sun + disk.x * tangent + disk.y * bitangent);
}

vec3 cosine_hemisphere_direction(vec3 normal, ivec2 coord, uint sample_index, uint sample_count, uint frame_number) {
    vec3 tangent;
    vec3 bitangent;
    orthonormal_basis(normal, tangent, bitangent);

    vec2 disk = vogel_disk_sample(1.0, int(sample_count), sample_seed(coord, sample_index, frame_number));
    float z = sqrt(max(0.0, 1.0 - dot(disk, disk)));

    return disk.x * tangent + disk.y * bitangent + z * normal;
}

#endif
