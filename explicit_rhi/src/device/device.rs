use crate::device::device_settings::DeviceSettings;
use anyhow::Result;

pub trait Device: Sized {
    fn create(settings: DeviceSettings) -> Result<Self>;

    fn destroy(self) -> Result<()>;
}
