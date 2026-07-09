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

vec2 vogelDiskSample(float diskRadius, int sampleCount, float seed) {
    float rotation = hash_unit(floatBitsToUint(seed) + 0x9e3779b9u) * TWO_PI;
    float radius = sqrt(seed / float(sampleCount)) * diskRadius;
    float theta = seed * GOLDEN_ANGLE + rotation;
    return radius * vec2(cos(theta), sin(theta));
}

#endif
