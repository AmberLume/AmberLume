#ifndef SAMPLERS_GLSL
#define SAMPLERS_GLSL

layout(set = 0, binding = 3) uniform sampler samplers[];

const uint SAMPLER_DEPTH = 0;
const uint SAMPLER_LINEAR_REPEAT = 1;
const uint SAMPLER_LINEAR_CLAMP = 2;
const uint SAMPLER_NEAREST_CLAMP = 3;
const uint SAMPLER_SHADOW = 4;

#endif
