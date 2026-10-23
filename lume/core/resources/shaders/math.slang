static const float PI = 3.14159265359;
static const float HALF_PI = 1.57079632679;
static const float TWO_PI = 6.283185307179586;
static const float GOLDEN_RATIO = 0.6180339887498948482;
static const float FLOAT_MAX = 3.402823466e+38;

float3x3 inverse(float3x3 value) {
    return spirv_asm {
        OpExtInst $$float3x3 result glsl450 MatrixInverse $value
    };
}

float3x3 normal_matrix(float4x4 transform) {
    float3x3 axes = transpose((float3x3)transform);
    float3x3 cofactor = float3x3(
        cross(axes[1], axes[2]),
        cross(axes[2], axes[0]),
        cross(axes[0], axes[1])
    );
    float handedness = dot(axes[0], cofactor[0]) < 0.0 ? -1.0 : 1.0;

    return transpose(cofactor) * handedness;
}
