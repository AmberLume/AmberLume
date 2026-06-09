#version 460
#extension GL_EXT_nonuniform_qualifier : enable

#include "push_constants.glsl"

layout(set = 0, binding = 0) uniform sampler2D scene[];

layout(location = 0) out vec4 out_color;

const mat3 LINEAR_SRGB_TO_LINEAR_REC2020 = mat3(
    vec3(0.6274, 0.0691, 0.0164),
    vec3(0.3293, 0.9195, 0.0880),
    vec3(0.0433, 0.0113, 0.8956)
);

const mat3 LINEAR_REC2020_TO_LINEAR_SRGB = mat3(
    vec3(1.6605, -0.1246, -0.0182),
    vec3(-0.5876, 1.1329, -0.1006),
    vec3(-0.0728, -0.0083, 1.1187)
);

const mat3 AGX_INSET = mat3(
    vec3(0.856627153315983, 0.137318972929847, 0.11189821299995),
    vec3(0.0951212405381588, 0.761241990602591, 0.0767994186031903),
    vec3(0.0482516061458583, 0.101439036467562, 0.811302368396859)
);

const mat3 AGX_OUTSET = mat3(
    vec3(1.1271005818144368, -0.1413297634984383, -0.14132976349843826),
    vec3(-0.11060664309660323, 1.157823702216272, -0.11060664309660294),
    vec3(-0.016493938717834573, -0.016493938717834257, 1.2519364065950405)
);

const float AGX_MIN_EV = -12.47393;
const float AGX_MAX_EV = 4.026069;

vec3 agx_contrast(vec3 x) {
    vec3 x2 = x * x;
    vec3 x4 = x2 * x2;
    return 15.5 * x4 * x2
        - 40.14 * x4 * x
        + 31.96 * x4
        - 6.868 * x2 * x
        + 0.4298 * x2
        + 0.1191 * x
        - 0.00232;
}

vec3 agx(vec3 color) {
    color = LINEAR_SRGB_TO_LINEAR_REC2020 * color;
    color = AGX_INSET * color;
    color = max(color, 1e-10);
    color = log2(color);
    color = (color - AGX_MIN_EV) / (AGX_MAX_EV - AGX_MIN_EV);
    color = clamp(color, 0.0, 1.0);
    color = agx_contrast(color);
    color = AGX_OUTSET * color;
    color = pow(max(color, vec3(0.0)), vec3(2.2));
    color = LINEAR_REC2020_TO_LINEAR_SRGB * color;
    color = clamp(color, 0.0, 1.0);
    return color;
}

void main() {
    ivec2 coord = ivec2(gl_FragCoord.xy);
    vec3 color = texelFetch(scene[nonuniformEXT(push_constants.input_texture)], coord, 0).rgb;

    color *= push_constants.exposure;
    color = agx(color);

    out_color = vec4(color, 1.0);
}
