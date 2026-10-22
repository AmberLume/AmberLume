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

float sample_seed(int2 coord, uint sample_index, uint frame_number) {
    uint pixel_hash = uint(coord.x) * 0x9e3779b9u
        ^ uint(coord.y) * 0x85ebca6bu
        ^ sample_index * 0xc2b2ae35u
        ^ frame_number * 0x27d4eb2fu;

    return float(sample_index) + hash_unit(pixel_hash);
}
