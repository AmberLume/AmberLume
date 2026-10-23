import "common.slang";

static const float RAY_T_MIN = 0.01;
static const float RAY_T_MAX = 10000.0;

bool trace_occluded(RaytracingAccelerationStructure acceleration_structure, float3 origin, float3 direction, float t_min, float t_max) {
    RayDesc ray;
    ray.Origin = origin;
    ray.TMin = t_min;
    ray.Direction = direction;
    ray.TMax = t_max;

    RayQuery<RAY_FLAG_ACCEPT_FIRST_HIT_AND_END_SEARCH | RAY_FLAG_FORCE_OPAQUE> query;
    query.TraceRayInline(acceleration_structure, RAY_FLAG_NONE, RT_INSTANCE_MASK_OPAQUE, ray);
    query.Proceed();

    return query.CommittedStatus() != COMMITTED_NOTHING;
}
