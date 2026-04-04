use ash::vk::{
    Extent3D, Format, ImageTiling, ImageType, ImageUsageFlags, SampleCountFlags, SharingMode,
};
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug)]
pub struct ImageDescription {
    pub image_type: ImageType,
    pub format: Format,
    pub extent: Extent3D,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub samples: SampleCountFlags,
    pub tiling: ImageTiling,
    pub usage: ImageUsageFlags,
    pub sharing_mode: SharingMode,
}

impl ImageDescription {
    pub fn default(format: Format, extent: Extent3D) -> Self {
        Self {
            image_type: ImageType::TYPE_2D,
            format,
            extent,
            mip_levels: 1,
            array_layers: 1,
            samples: SampleCountFlags::TYPE_1,
            tiling: ImageTiling::OPTIMAL,
            usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
            sharing_mode: SharingMode::EXCLUSIVE,
        }
    }
}

impl Hash for ImageDescription {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self {
            image_type,
            format,
            extent,
            mip_levels,
            array_layers,
            samples,
            tiling,
            usage,
            sharing_mode,
        } = self;

        image_type.as_raw().hash(state);
        format.as_raw().hash(state);

        extent.width.hash(state);
        extent.height.hash(state);
        extent.depth.hash(state);

        mip_levels.hash(state);
        array_layers.hash(state);

        samples.as_raw().hash(state);
        tiling.as_raw().hash(state);
        usage.as_raw().hash(state);
        sharing_mode.as_raw().hash(state);
    }
}
