import "math.slang";

static const float GOLDEN_ANGLE = 2.399963229728653;

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

float2 vogel_disk_sample(float disk_radius, int sample_count, float seed) {
    float rotation = hash_unit(asuint(seed) + 0x9e3779b9u) * TWO_PI;
    float radius = sqrt(seed / float(sample_count)) * disk_radius;
    float theta = seed * GOLDEN_ANGLE + rotation;
    return radius * float2(cos(theta), sin(theta));
}

float sample_seed(int2 coord, uint sample_index, uint frame_number) {
    uint pixel_hash = uint(coord.x) * 0x9e3779b9u
        ^ uint(coord.y) * 0x85ebca6bu
        ^ sample_index * 0xc2b2ae35u
        ^ frame_number * 0x27d4eb2fu;

    return float(sample_index) + hash_unit(pixel_hash);
}

void orthonormal_basis(float3 axis, out float3 tangent, out float3 bitangent) {
    float3 up = abs(axis.y) < 0.999 ? float3(0.0, 1.0, 0.0) : float3(1.0, 0.0, 0.0);

    tangent = normalize(cross(up, axis));
    bitangent = cross(axis, tangent);
}

float3 sun_disk_direction(float3 to_sun, float angular_radius, int2 coord, uint sample_index, uint sample_count, uint frame_number) {
    float3 tangent;
    float3 bitangent;
    orthonormal_basis(to_sun, tangent, bitangent);

    float2 disk = vogel_disk_sample(tan(angular_radius), int(sample_count), sample_seed(coord, sample_index, frame_number));

    return normalize(to_sun + disk.x * tangent + disk.y * bitangent);
}

float3 cosine_hemisphere_direction(float3 normal, int2 coord, uint sample_index, uint sample_count, uint frame_number) {
    float3 tangent;
    float3 bitangent;
    orthonormal_basis(normal, tangent, bitangent);

    float2 disk = vogel_disk_sample(1.0, int(sample_count), sample_seed(coord, sample_index, frame_number));
    float z = sqrt(max(0.0, 1.0 - dot(disk, disk)));

    return disk.x * tangent + disk.y * bitangent + z * normal;
}
