import "../common.slang";
import "../aabb.slang";
import "../math.slang";

struct MeshBinding {
    float4x4 inverse_bind_matrix;

    float4 bounds_min;
    float4 bounds_max;
};

struct Animation {
    uint offset;
    uint bone_count;
    uint frame_count;
    float duration;
    float fps;
    uint _pad0[3];
};

struct AnimationFrame {
    float3 translation;
    uint _pad0;
    float4 rotation;
    float3 scale;
    uint _pad1;
};

struct Skeleton {
    uint bone_offset;
    uint bone_count;
    uint _pad0[2];
};

struct SkeletonBone {
    int parent_index;
    uint _pad0[3];
};

struct BonePose {
    float3 translation;
    float4 rotation;
    float3 scale;
};

struct PushConstants {
    Ptr<SkinningInstance, Access.Read> skinning_instance_buffer_device_address;
    Ptr<Animation, Access.Read> animation_buffer_device_address;
    Ptr<AnimationFrame, Access.Read> animation_frame_buffer_device_address;
    Ptr<Skeleton, Access.Read> skeleton_buffer_device_address;
    Ptr<SkeletonBone, Access.Read> skeleton_bone_buffer_device_address;
    Ptr<Mesh, Access.Read> mesh_buffer_device_address;
    Ptr<MeshBinding, Access.Read> mesh_binding_buffer_device_address;
    Ptr<SubmeshBounds> submesh_bounds_buffer_device_address;
    Ptr<BoneTransform> bone_transform_buffer_device_address;
    uint instance_count;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

float4x4 trs(BonePose pose) {
    float3 t = pose.translation;
    float3 s = pose.scale;
    float x = pose.rotation.x;
    float y = pose.rotation.y;
    float z = pose.rotation.z;
    float w = pose.rotation.w;

    return float4x4(
        float4(s.x * (1.0 - 2.0 * (y * y + z * z)), s.y * (2.0 * (x * y - z * w)), s.z * (2.0 * (x * z + y * w)), t.x),
        float4(s.x * (2.0 * (x * y + z * w)), s.y * (1.0 - 2.0 * (x * x + z * z)), s.z * (2.0 * (y * z - x * w)), t.y),
        float4(s.x * (2.0 * (x * z - y * w)), s.y * (2.0 * (y * z + x * w)), s.z * (1.0 - 2.0 * (x * x + y * y)), t.z),
        float4(0.0, 0.0, 0.0, 1.0)
    );
}

float4 quat_slerp(float4 a, float4 b, float t) {
    float d = dot(a, b);

    if (d < 0.0) {
        b = -b;
        d = -d;
    }
    if (d > 0.9995) {
        return normalize(lerp(a, b, t));
    }

    float theta_0 = acos(d);
    float theta = theta_0 * t;
    float s0 = sin(theta_0 - theta) / sin(theta_0);
    float s1 = sin(theta) / sin(theta_0);

    return a * s0 + b * s1;
}

BonePose sample_bone(Animation animation, float time, uint bone_index) {
    float clamped_time = clamp(time, 0.0, animation.duration - 0.0001);
    float frame = clamped_time * animation.fps;
    uint frame_a = uint(frame);
    uint frame_b = min(frame_a + 1, animation.frame_count - 1);
    float alpha = frac(frame);

    Ptr<AnimationFrame, Access.Read> frames = push_constants.animation_frame_buffer_device_address + animation.offset + bone_index * animation.frame_count;

    AnimationFrame a = frames[frame_a];
    AnimationFrame b = frames[frame_b];

    BonePose pose;
    pose.translation = lerp(a.translation, b.translation, alpha);
    pose.rotation = quat_slerp(a.rotation, b.rotation, alpha);
    pose.scale = lerp(a.scale, b.scale, alpha);

    return pose;
}

BonePose blend_bone_pose(BonePose from, BonePose to, float factor) {
    BonePose pose;
    pose.translation = lerp(from.translation, to.translation, factor);
    pose.rotation = quat_slerp(from.rotation, to.rotation, factor);
    pose.scale = lerp(from.scale, to.scale, factor);

    return pose;
}

void write_bone_world_transforms(SkinningPose pose, Skeleton skeleton, Ptr<BoneTransform> transforms) {
    Animation animation = push_constants.animation_buffer_device_address[pose.animation_id];

    bool blending = pose.blend_factor < 1.0;

    Animation blend_from_animation = animation;
    if (blending) {
        blend_from_animation = push_constants.animation_buffer_device_address[pose.blend_from_animation_id];
    }

    for (uint i = 0; i < skeleton.bone_count; i++) {
        BonePose bone_pose = sample_bone(animation, pose.time, i);

        if (blending) {
            bone_pose = blend_bone_pose(sample_bone(blend_from_animation, pose.blend_from_time, i), bone_pose, pose.blend_factor);
        }

        float4x4 local_transform = trs(bone_pose);
        SkeletonBone bone = push_constants.skeleton_bone_buffer_device_address[skeleton.bone_offset + i];

        transforms[i].transform = bone.parent_index < 0
            ? local_transform
            : mul(transforms[uint(bone.parent_index)].transform, local_transform);
    }
}

void compute_skin_bounds(Skeleton skeleton, Ptr<MeshBinding, Access.Read> mesh_bindings, Ptr<BoneTransform> transforms, out float3 bounds_min, out float3 bounds_max) {
    bounds_min = float3(FLOAT_MAX);
    bounds_max = float3(-FLOAT_MAX);

    for (uint i = 0; i < skeleton.bone_count; i++) {
        MeshBinding mesh_binding = mesh_bindings[i];

        if (any(mesh_binding.bounds_min.xyz > mesh_binding.bounds_max.xyz)) {
            continue;
        }

        float3 bone_bounds_min;
        float3 bone_bounds_max;
        transform_aabb(transforms[i].transform, mesh_binding.bounds_min.xyz, mesh_binding.bounds_max.xyz, bone_bounds_min, bone_bounds_max);

        bounds_min = min(bounds_min, bone_bounds_min);
        bounds_max = max(bounds_max, bone_bounds_max);
    }
}

void apply_inverse_binds(Skeleton skeleton, Ptr<MeshBinding, Access.Read> mesh_bindings, Ptr<BoneTransform> transforms) {
    for (uint i = 0; i < skeleton.bone_count; i++) {
        transforms[i].transform = mul(transforms[i].transform, mesh_bindings[i].inverse_bind_matrix);
    }
}

[shader("compute")]
[numthreads(256, 1, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    uint instance_id = global_invocation_id.x;
    if (instance_id >= push_constants.instance_count) {
        return;
    }

    SkinningInstance instance = push_constants.skinning_instance_buffer_device_address[instance_id];
    Skeleton skeleton = push_constants.skeleton_buffer_device_address[instance.skeleton_id];
    Mesh mesh = push_constants.mesh_buffer_device_address[instance.mesh_id];

    Ptr<BoneTransform> transforms = push_constants.bone_transform_buffer_device_address + instance.bone_transform_offset;
    Ptr<MeshBinding, Access.Read> mesh_bindings = push_constants.mesh_binding_buffer_device_address + mesh.binding_offset;

    write_bone_world_transforms(instance.pose, skeleton, transforms);

    float3 bounds_min;
    float3 bounds_max;
    compute_skin_bounds(skeleton, mesh_bindings, transforms, bounds_min, bounds_max);

    apply_inverse_binds(skeleton, mesh_bindings, transforms);

    push_constants.submesh_bounds_buffer_device_address[instance.bounds_index] = { bounds_min, bounds_max };
}
