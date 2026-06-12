#ifndef BINDINGS_GLSL
#define BINDINGS_GLSL

#extension GL_EXT_nonuniform_qualifier : enable

#define GLOBAL_SET 0

#define BINDING_SAMPLER 0
#define BINDING_TEXTURE 1
#define BINDING_SHADOW_ARRAY 2
#define BINDING_STORAGE_IMAGE 3
#define BINDING_GRAPH_TEXTURE 4

const uint SAMPLER_DEPTH = 0;
const uint SAMPLER_LINEAR_REPEAT = 1;
const uint SAMPLER_LINEAR_CLAMP = 2;
const uint SAMPLER_NEAREST_CLAMP = 3;
const uint SAMPLER_SHADOW = 4;

layout(set = GLOBAL_SET, binding = BINDING_SAMPLER) uniform sampler samplers[];
layout(set = GLOBAL_SET, binding = BINDING_TEXTURE) uniform texture2D textures[];
layout(set = GLOBAL_SET, binding = BINDING_TEXTURE) uniform utexture2D utextures[];
layout(set = GLOBAL_SET, binding = BINDING_SHADOW_ARRAY) uniform texture2DArray shadow_arrays[];

#endif
