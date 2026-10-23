static const float PI = 3.14159265359;
static const float HALF_PI = 1.57079632679;

float4x4 inverse(float4x4 value) {
    return spirv_asm {
        OpExtInst $$float4x4 result glsl450 MatrixInverse $value
    };
}
