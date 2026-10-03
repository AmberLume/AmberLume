use gpu_data::SkeletonBoneGPU;

#[derive(Clone)]
pub struct SkeletonConfig {
    pub name: String,

    pub bones: Vec<SkeletonBoneGPU>,
}
