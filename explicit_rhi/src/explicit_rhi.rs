use crate::device::device::Device;

pub trait ExplicitRhi {
    type Device: Device;
}
