#ifndef SKY_GLSL
#define SKY_GLSL

vec3 sky_hash33(vec3 p) {
    p = vec3(
        dot(p, vec3(127.1, 311.7, 74.7)),
        dot(p, vec3(269.5, 183.3, 246.1)),
        dot(p, vec3(113.5, 271.9, 124.6))
    );
    return fract(sin(p) * 43758.5453123);
}

vec3 sky_stars(vec3 dir, float time) {
    const float frequency = 50.0;
    const float probability = 0.45;
    const float core_size = 0.06;
    const vec3 color_cool = vec3(0.78, 0.86, 1.0);
    const vec3 color_warm = vec3(1.0, 0.9, 0.78);

    vec3 p = dir * frequency;
    vec3 id = floor(p);

    vec3 r1 = sky_hash33(id);
    if (r1.x > probability) {
        return vec3(0.0);
    }

    vec3 r2 = sky_hash33(id * 1.37 + 7.0);

    vec3 center = id + 0.5 + (r1.yzx - 0.5) * 0.7;
    float dist = length(p - center);

    float size = core_size * mix(0.8, 1.2, r2.x);
    float core = smoothstep(size, 0.0, dist);

    float phase = r1.y * 6.2831853;
    float speed = mix(1.5, 5.0, r1.z);
    float t = time * speed + phase;
    float twinkle = 0.35 + 0.65 * (0.5 + 0.5 * sin(t));
    twinkle += 0.6 * pow(0.5 + 0.5 * sin(t * 1.7 + phase), 8.0);

    vec3 base = mix(color_cool, color_warm, r2.y);
    vec3 tint = mix(vec3(1.0), base, 0.55);

    return tint * core * twinkle;
}

vec3 procedural_sky(vec3 dir, vec3 sun_dir, float time, bool detail) {
    const vec3 zenith_day = vec3(0.08, 0.22, 0.52);
    const vec3 horizon_day = vec3(0.52, 0.62, 0.74);
    const vec3 horizon_sunset = vec3(1.0, 0.42, 0.14);
    const vec3 zenith_night = vec3(0.004, 0.008, 0.018);
    const vec3 horizon_night = vec3(0.01, 0.016, 0.028);

    float sun_elev = sun_dir.y;
    float sun_dot = dot(dir, sun_dir);

    float day = smoothstep(-0.12, 0.18, sun_elev);
    float sunset = exp(-max(sun_elev, 0.0) * 7.0) * smoothstep(-0.18, 0.04, sun_elev);

    vec2 dir_azimuth = dir.xz;
    vec2 sun_azimuth = sun_dir.xz;
    float dir_azimuth_len = length(dir_azimuth);
    float sun_azimuth_len = length(sun_azimuth);
    float azimuth_align = (dir_azimuth_len > 1e-4 && sun_azimuth_len > 1e-4)
        ? dot(dir_azimuth, sun_azimuth) / (dir_azimuth_len * sun_azimuth_len)
        : 0.0;
    float sun_side = pow(max(azimuth_align, 0.0), 2.0);

    vec3 zenith = mix(zenith_night, zenith_day, day);
    vec3 horizon = mix(horizon_night, horizon_day, day);
    horizon = mix(horizon, horizon_sunset, sunset * sun_side);

    float h = clamp(dir.y, 0.0, 1.0);
    float grad = pow(1.0 - h, 2.5);
    vec3 sky = mix(zenith, horizon, grad);

    if (detail) {
        float glow = pow(max(sun_dot, 0.0), 5.0);
        sky += horizon_sunset * glow * sunset * 0.9;

        float halo = pow(max(sun_dot, 0.0), 8.0);
        sky += vec3(1.0, 0.92, 0.78) * halo * day * 0.35;

        float disk = smoothstep(0.9997, 0.9999, sun_dot);
        float sun_visible = smoothstep(-0.04, 0.02, sun_elev);
        vec3 sun_color = mix(vec3(1.0, 0.96, 0.88), vec3(1.0, 0.5, 0.22), sunset);
        sky += sun_color * disk * sun_visible * 18.0;
    }

    sky += sky_stars(dir, time) * (1.0 - day);

    float below = smoothstep(0.0, -0.12, dir.y);
    sky = mix(sky, sky * 0.25, below);

    return sky;
}

#endif
