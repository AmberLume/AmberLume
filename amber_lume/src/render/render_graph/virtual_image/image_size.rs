use ash::vk::Extent2D;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageSize {
    Swapchain { pow: u32 },
}

impl ImageSize {
    pub fn full_swapchain() -> Self {
        Self::Swapchain { pow: 0 }
    }

    pub fn resolve(self, swapchain_extent: Extent2D) -> Extent2D {
        match self {
            ImageSize::Swapchain { pow } => Extent2D {
                width: (swapchain_extent.width >> pow).max(1),
                height: (swapchain_extent.height >> pow).max(1),
            },
        }
    }
}
