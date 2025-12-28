#version 460

layout(location = 0) in vec3 fragNormal;
layout(location = 0) out vec4 out_color;

void main() {
    vec3 normal = normalize(fragNormal);

    vec3 keyLight = normalize(vec3(0.5, 1.0, 0.5));
    vec3 fillLight = normalize(vec3(-0.3, 0.2, 0.8));

    float keyDiffuse = max(dot(normal, keyLight), 0.0);
    float fillDiffuse = max(dot(normal, fillLight), 0.0);

    float ambient = 0.3;
    float lighting = ambient + keyDiffuse * 0.5 + fillDiffuse * 0.3;

    vec3 baseColor = vec3(0.9, 0.3, 0.3);
    out_color = vec4(normal * 0.5 + 0.5, 1.0);
}
