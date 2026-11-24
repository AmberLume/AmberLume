use crate::device::device::Device;
use explicit_rhi::ExplicitRhi;

pub struct Vulkan;

impl ExplicitRhi for Vulkan {
    type Device = Device;
}
