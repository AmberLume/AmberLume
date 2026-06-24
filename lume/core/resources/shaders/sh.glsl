#ifndef SH_GLSL
#define SH_GLSL

void sh_basis(vec3 d, out float y[9]) {
    y[0] = 0.282095;
    y[1] = 0.488603 * d.y;
    y[2] = 0.488603 * d.z;
    y[3] = 0.488603 * d.x;
    y[4] = 1.092548 * d.x * d.y;
    y[5] = 1.092548 * d.y * d.z;
    y[6] = 0.315392 * (3.0 * d.z * d.z - 1.0);
    y[7] = 1.092548 * d.x * d.z;
    y[8] = 0.546274 * (d.x * d.x - d.y * d.y);
}

vec3 sh_irradiance(vec3 c[9], vec3 n) {
    float y[9];
    sh_basis(n, y);

    vec3 result = c[0] * y[0];
    for (int i = 1; i < 4; i++) {
        result += (2.0 / 3.0) * c[i] * y[i];
    }
    for (int i = 4; i < 9; i++) {
        result += (1.0 / 4.0) * c[i] * y[i];
    }

    return result;
}

#endif
