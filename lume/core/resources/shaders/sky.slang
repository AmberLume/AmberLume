float3 sky_hash33(float3 p) {
    p = float3(
        dot(p, float3(127.1, 311.7, 74.7)),
        dot(p, float3(269.5, 183.3, 246.1)),
        dot(p, float3(113.5, 271.9, 124.6))
    );
    return frac(sin(p) * 43758.5453123);
}

float3 sky_stars(float3 direction, float time) {
    const float frequency = 50.0;
    const float probability = 0.45;
    const float core_size = 0.06;
    const float3 color_cool = float3(0.78, 0.86, 1.0);
    const float3 color_warm = float3(1.0, 0.9, 0.78);

    float3 p = direction * frequency;
    float3 id = floor(p);

    float3 r1 = sky_hash33(id);
    if (r1.x > probability) {
        return float3(0.0);
    }

    float3 r2 = sky_hash33(id * 1.37 + 7.0);

    float3 center = id + 0.5 + (r1.yzx - 0.5) * 0.7;
    float center_distance = length(p - center);

    float size = core_size * lerp(0.8, 1.2, r2.x);
    float core = smoothstep(size, 0.0, center_distance);

    float phase = r1.y * 6.2831853;
    float speed = lerp(1.5, 5.0, r1.z);
    float t = time * speed + phase;
    float twinkle = 0.35 + 0.65 * (0.5 + 0.5 * sin(t));
    twinkle += 0.6 * pow(0.5 + 0.5 * sin(t * 1.7 + phase), 8.0);

    float3 base = lerp(color_cool, color_warm, r2.y);
    float3 tint = lerp(float3(1.0), base, 0.55);

    return tint * core * twinkle;
}

static const float3 SKY_HORIZON_SUNSET = float3(1.0, 0.42, 0.14);

float sky_day(float sun_elevation) {
    return smoothstep(-0.12, 0.18, sun_elevation);
}

float sky_sunset(float sun_elevation) {
    return exp(-max(sun_elevation, 0.0) * 7.0) * smoothstep(-0.18, 0.04, sun_elevation);
}

float sky_ground_darkening(float3 direction) {
    return lerp(1.0, 0.25, smoothstep(0.0, -0.12, direction.y));
}

float3 procedural_sky(float3 direction, float3 sun_direction, float time) {
    const float3 zenith_day = float3(0.08, 0.22, 0.52);
    const float3 horizon_day = float3(0.52, 0.62, 0.74);
    const float3 zenith_night = float3(0.004, 0.008, 0.018);
    const float3 horizon_night = float3(0.01, 0.016, 0.028);

    float day = sky_day(sun_direction.y);
    float sunset = sky_sunset(sun_direction.y);

    float2 direction_azimuth = direction.xz;
    float2 sun_azimuth = sun_direction.xz;
    float direction_azimuth_length = length(direction_azimuth);
    float sun_azimuth_length = length(sun_azimuth);
    float azimuth_align = (direction_azimuth_length > 1e-4 && sun_azimuth_length > 1e-4)
        ? dot(direction_azimuth, sun_azimuth) / (direction_azimuth_length * sun_azimuth_length)
        : 0.0;
    float sun_side = pow(max(azimuth_align, 0.0), 2.0);

    float3 zenith = lerp(zenith_night, zenith_day, day);
    float3 horizon = lerp(horizon_night, horizon_day, day);
    horizon = lerp(horizon, SKY_HORIZON_SUNSET, sunset * sun_side);

    float h = clamp(direction.y, 0.0, 1.0);
    float gradient = pow(1.0 - h, 2.5);
    float3 sky = lerp(zenith, horizon, gradient);

    sky += sky_stars(direction, time) * (1.0 - day);

    return sky * sky_ground_darkening(direction);
}

float3 procedural_sun(float3 direction, float3 sun_direction) {
    float day = sky_day(sun_direction.y);
    float sunset = sky_sunset(sun_direction.y);
    float sun_dot = dot(direction, sun_direction);

    float glow = pow(max(sun_dot, 0.0), 5.0);
    float halo = pow(max(sun_dot, 0.0), 8.0);
    float disk = smoothstep(0.9997, 0.9999, sun_dot);
    float sun_visible = smoothstep(-0.04, 0.02, sun_direction.y);
    float3 sun_color = lerp(float3(1.0, 0.96, 0.88), float3(1.0, 0.5, 0.22), sunset);

    float3 sun = SKY_HORIZON_SUNSET * glow * sunset * 0.9;
    sun += float3(1.0, 0.92, 0.78) * halo * day * 0.35;
    sun += sun_color * disk * sun_visible * 18.0;

    return sun * sky_ground_darkening(direction);
}
