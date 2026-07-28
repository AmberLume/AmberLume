#version 460

layout(location = 0) in flat uint entity_index;

layout(location = 0) out uint out_entity_index;

void main() {
    out_entity_index = entity_index;
}
