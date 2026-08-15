#version 460

layout(location = 0) in flat uint entity_index;
layout(location = 1) in vec4 current_clip;
layout(location = 2) in vec4 previous_clip;

layout(location = 0) out uint out_entity_index;
layout(location = 1) out vec2 out_velocity;

void main() {
    out_entity_index = entity_index;

    vec2 current_ndc = current_clip.xy / current_clip.w;
    vec2 previous_ndc = previous_clip.xy / previous_clip.w;
    out_velocity = (previous_ndc - current_ndc) * 0.5;
}
