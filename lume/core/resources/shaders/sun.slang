import "common.slang";

float3 sun_irradiance(Ptr<Scene, Access.Read> scene, float n_dot_l) {
    float above_horizon = smoothstep(-0.05, 0.02, -scene.light_direction.y);

    return scene.light_color * scene.light_intensity * n_dot_l * above_horizon;
}
