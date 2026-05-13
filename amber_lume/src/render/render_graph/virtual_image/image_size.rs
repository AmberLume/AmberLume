use ash::vk::Extent2D;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageSize {
    Target { pow: u32 },
}

impl ImageSize {
    pub fn full() -> Self {
        Self::Target { pow: 0 }
    }

    pub fn resolve(self, target_extent: Extent2D) -> Extent2D {
        match self {
            ImageSize::Target { pow } => Extent2D {
                width: (target_extent.width >> pow).max(1),
                height: (target_extent.height >> pow).max(1),
            },
        }
    }
}
