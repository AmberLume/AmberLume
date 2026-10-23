static const uint GLOBAL_SET = 0;

static const uint BINDING_SAMPLER = 0;
static const uint BINDING_TEXTURE = 1;
static const uint BINDING_STORAGE_IMAGE = 3;
static const uint BINDING_GRAPH_TEXTURE = 4;
static const uint BINDING_ACCELERATION_STRUCTURE = 5;

static const uint SAMPLER_LINEAR_CLAMP = 2;
static const uint SAMPLER_NEAREST_CLAMP = 3;
static const uint SAMPLER_SHADOW = 4;

[[vk::binding(BINDING_SAMPLER, GLOBAL_SET)]] SamplerState samplers[];
[[vk::binding(BINDING_SAMPLER, GLOBAL_SET)]] SamplerComparisonState comparison_samplers[];
[[vk::binding(BINDING_TEXTURE, GLOBAL_SET)]] Texture2D textures[];
[[vk::binding(BINDING_STORAGE_IMAGE, GLOBAL_SET)]] [[vk::image_format("r16f")]] RWTexture2D<float> storage_images_r16f[];
[[vk::binding(BINDING_STORAGE_IMAGE, GLOBAL_SET)]] [[vk::image_format("rgba16f")]] RWTexture2D<float4> storage_images_rgba16f[];
[[vk::binding(BINDING_STORAGE_IMAGE, GLOBAL_SET)]] [[vk::image_format("r32ui")]] RWTexture2D<uint> storage_images_r32ui[];
[[vk::binding(BINDING_GRAPH_TEXTURE, GLOBAL_SET)]] Texture2D graph_textures[];
[[vk::binding(BINDING_GRAPH_TEXTURE, GLOBAL_SET)]] [[vk::image_format("unknown")]] Texture2D<uint> graph_utextures[];
[[vk::binding(BINDING_GRAPH_TEXTURE, GLOBAL_SET)]] Texture2DArray graph_texture_arrays[];
[[vk::binding(BINDING_ACCELERATION_STRUCTURE, GLOBAL_SET)]] RaytracingAccelerationStructure acceleration_structures[];
